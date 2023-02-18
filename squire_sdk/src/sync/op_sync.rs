use std::{cell::RefCell, cmp::min};

use serde::{Deserialize, Serialize};

use crate::model::{
    accounts::SquireAccount,
    identifiers::{PlayerId, RoundId},
    operations::OpUpdate,
    tournament::{Tournament, TournamentSeed},
};

use super::{op_log::OpSlice, sync_status::SyncStatus, sync_error::SyncError, FullOp, OpDiff, Blockage, OpId, OpLog};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A struct to help resolve syncing op logs
pub struct OpSync {
    pub(crate) owner: SquireAccount,
    pub(crate) seed: TournamentSeed,
    pub(crate) ops: OpSlice,
}

impl OpSync {
    /// Calculates the length of inner `Vec` of `FullOps`
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if the length of inner `Vec` of `FullOp`s is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
    
    /// Slipts an `OpLog` into two pieces based off the first operation in this sync. The first
    /// operation of this sync will be the first operation of the second half. If this log is empty
    /// or the first operation can't be found, an error is returned.
    pub fn bisect_log(&self, log: &OpLog) -> Result<(OpSlice, OpSlice), SyncError> {
        let id = self.first_id()?;
        let slices = log.split_at(id);
        if slices.1.is_empty() {
            Err(SyncError::UnknownOperation(Box::new(self.first_op()?)))
        } else {
            Ok(slices)
        }
    }
    
    /// Returns the first operation, if it exists. Otherwise, a `SyncError::EmptySync` is
    /// returned.
    pub fn first_op(&self) -> Result<FullOp, SyncError> {
        self.ops.start_op().ok_or(SyncError::EmptySync)
    }
    
    /// Returns the first operation's id, if it exists. Otherwise, a `SyncError::EmptySync` is
    /// returned.
    pub fn first_id(&self) -> Result<OpId, SyncError> {
        self.ops.start_id().ok_or(SyncError::EmptySync)
    }
}

pub(crate) fn process_sync(
    base: &mut Tournament,
    mut known: OpSlice,
    mut known_in_progress: OpSlice,
    other: OpSlice,
    mut accepted: OpSlice,
) -> SyncStatus {
    // Inner tuples are (old, new)
    let player_updates = RefCell::new(Vec::<(PlayerId, PlayerId)>::new());
    // Inner tuples are (old, new)
    let round_updates = RefCell::new(Vec::<(RoundId, RoundId)>::new());
    let mut iter = other.ops.into_iter().map(|mut op| {
        player_updates
            .borrow()
            .iter()
            .for_each(|(old, new)| op.swap_player_ids(*old, *new));
        round_updates
            .borrow()
            .iter()
            .for_each(|(old, new)| op.swap_round_ids(*old, *new));
        op
    });
    let time_update = |old, new| match (old, new) {
        (OpUpdate::None, OpUpdate::None) => {}
        (OpUpdate::PlayerId(old), OpUpdate::PlayerId(new)) => {
            player_updates.borrow_mut().push((old, new));
        }
        (OpUpdate::RoundId(old), OpUpdate::RoundId(new)) => {
            assert_eq!(old.len(), new.len());
            round_updates
                .borrow_mut()
                .extend(old.into_iter().zip(new.into_iter()));
        }
        _ => {
            unreachable!(
                "The inner operations are identical, so their updates should be the same variant."
            );
        }
    };
    // Iterated through the other ops
    // For each op, iterate through known_in_progress
    for op in iter.by_ref() {
        match process_sync_inner(base, &op, known.ops.iter(), time_update) {
            SyncInnerStatus::Passes(Some(i)) => {
                let len = min(known_in_progress.len() - 1, i);
                known_in_progress.ops.drain(0..=len);
            }
            SyncInnerStatus::Passes(None) => {
                accepted.ops.push(op);
            }
            SyncInnerStatus::Blockage(blocks) => {
                return SyncStatus::InProgress(Box::new(Blockage {
                    base: base.clone(),
                    known,
                    known_in_progress,
                    accepted,
                    other: iter.collect::<Vec<_>>().into(),
                    problem: op,
                    blocks,
                }));
            }
            SyncInnerStatus::RollbackFound => {
                return SyncStatus::SyncError(Box::new(SyncError::RollbackFound(
                    known_in_progress,
                )));
            }
        }
    }
    // All the agreed upon operations happen after this log
    known.ops.extend(accepted.ops);
    SyncStatus::Completed(OpSync { ops: known })
}

pub(crate) enum SyncInnerStatus {
    Passes(Option<usize>),
    Blockage(usize),
    RollbackFound,
}

pub(crate) fn process_sync_inner<'a, I, F>(
    base: &mut Tournament,
    other: &'a FullOp,
    known: I,
    mut time_update: F,
) -> SyncInnerStatus
where
    I: Iterator<Item = &'a FullOp>,
    F: FnMut(OpUpdate, OpUpdate),
{
    for (i, known_op) in known.enumerate() {
        match other.diff(known_op) {
            OpDiff::Different => {
                todo!("Implement tournament cursor/simulation method of checking for blockages");
            }
            OpDiff::Inactive => {
                return SyncInnerStatus::RollbackFound;
            }
            OpDiff::Time => {
                // Identical ops at different times, case by case
                // Could require changing the round/player ids in the rest of the other
                // operations and remove this known_op from known_in_progress (like
                // registering a guest)
                // Could just require this known_op to be removed from known_in_progress
                // (like recording a match result)
                time_update(other.get_update(), known_op.get_update());
                return SyncInnerStatus::Passes(Some(i));
            }
            OpDiff::Equal => {
                return SyncInnerStatus::Passes(Some(i));
            }
        }
    }
    SyncInnerStatus::Passes(None)
}
