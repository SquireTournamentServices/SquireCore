use std::{cell::RefCell, cmp::min};

use serde::{Deserialize, Serialize};

use crate::{
    identifiers::{PlayerId, RoundId},
    operations::{FullOp, OpSlice},
};

use super::{OpDiff, OpUpdate};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A struct to help resolve syncing op logs
pub struct OpSync {
    pub(crate) ops: OpSlice,
}

impl From<OpSlice> for OpSync {
    fn from(ops: OpSlice) -> Self {
        Self { ops }
    }
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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An enum to help track the progress of the syncing of two op logs
pub enum SyncStatus {
    /// There was an error when attempting to initially sync
    SyncError(Box<SyncError>),
    /// There are discrepancies in between the two logs that are being synced
    InProgress(Box<Blockage>),
    /// The logs have been successfully synced
    Completed(OpSync),
}

impl SyncStatus {
    /// Calculates if the status is an error
    pub fn is_error(&self) -> bool {
        matches!(self, SyncStatus::SyncError(_))
    }

    /// Calculates if the status is a blockage
    pub fn is_in_progress(&self) -> bool {
        matches!(self, SyncStatus::InProgress(_))
    }

    /// Calculates if the status is a success
    pub fn is_completed(&self) -> bool {
        matches!(self, SyncStatus::Completed(_))
    }

    /// Comsumes self and returns the held error if `self` is `SyncError` and panics otherwise
    pub fn assume_error(self) -> Box<SyncError> {
        match self {
            SyncStatus::SyncError(err) => err,
            SyncStatus::InProgress(block) => {
                panic!("Sync status was not an error but was a blockage: {block:?}")
            }
            SyncStatus::Completed(sync) => {
                panic!("Sync status was not an error but was completed: {sync:?}")
            }
        }
    }

    /// Comsumes self and returns the held error if `self` is `InProgress` and panics otherwise
    pub fn assume_in_progress(self) -> Box<Blockage> {
        match self {
            SyncStatus::InProgress(block) => block,
            SyncStatus::SyncError(err) => {
                panic!("Sync status was not a blockage but was an error: {err:?}")
            }
            SyncStatus::Completed(sync) => {
                panic!("Sync status was not a blockage but was completed: {sync:?}")
            }
        }
    }

    /// Comsumes self and returns the held error if `self` is `Complete` and panics otherwise
    pub fn assume_completed(self) -> OpSync {
        match self {
            SyncStatus::Completed(sync) => sync,
            SyncStatus::InProgress(block) => {
                panic!("Sync status was not completed but was a blockage: {block:?}")
            }
            SyncStatus::SyncError(err) => {
                panic!("Sync status was not completed but was an error: {err:?}")
            }
        }
    }
}

/// An enum to that captures the error that might occur when sync op logs.
/// `UnknownOperation` encodes that first operation in an OpSlice is unknown
/// `RollbackFound` encode that a rollback has occured remotely but not locally and returns an
/// OpSlice that contains everything since that rollback. When recieved, this new log should
/// overwrite the local log
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// At least one of the logs was empty
    EmptySync,
    /// The tournament logs were merged, but an operation is causing an error in the tournament
    /// itself. Contains the operation that is causing the problem and the merged log
    FailedSync(Box<(FullOp, OpSync)>),
    /// The starting operation of the slice in unknown to the other log
    UnknownOperation(Box<FullOp>),
    /// One of the logs contains a rollback that the other doesn't have
    RollbackFound(OpSlice),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A struct to help resolve blockages
pub struct Blockage {
    pub(crate) known: OpSlice,
    pub(crate) known_in_progress: OpSlice,
    pub(crate) accepted: OpSlice,
    pub(crate) other: OpSlice,
    pub(crate) problem: FullOp,
    /// The index into `known_in_progress` that contains the blocking operation
    pub(crate) blocks: usize,
}

/// Holds references to the operations that are causing a blockage in the sync process
#[derive(Debug, Clone)]
pub struct Problem<'a> {
    /// The operation that is trying to be added to the log, but is causing problems
    pub other: &'a FullOp,
    /// The operation that is canon that is blocking the `other` operation
    pub known: &'a FullOp,
}

impl Blockage {
    /// Returns the problematic pair of operations.
    pub fn problem(&self) -> Problem<'_> {
        Problem {
            other: &self.problem,
            known: &self.known_in_progress.ops[self.blocks],
        }
    }

    /// Ignores the problematic operation and continues the syncing process
    pub fn ignore(self) -> SyncStatus {
        let Self {
            known,
            known_in_progress,
            accepted,
            other,
            ..
        } = self;
        process_sync(known, known_in_progress, other, accepted)
    }

    /// Accepts that the problematic operation is meant to occur after the operation it blocks and
    /// continues the syncing process
    pub fn push(mut self) -> SyncStatus {
        let Self {
            known,
            mut known_in_progress,
            mut accepted,
            mut other,
            problem,
            blocks,
        } = self;
        let mut player_updates = Vec::new();
        let mut round_updates = Vec::new();
        let time_update = |old, new| match (old, new) {
            (OpUpdate::None, OpUpdate::None) => {}
            (OpUpdate::PlayerId(old), OpUpdate::PlayerId(new)) => {
                player_updates.push((old, new));
            }
            (OpUpdate::RoundId(old), OpUpdate::RoundId(new)) => {
                assert_eq!(old.len(), new.len());
                round_updates.extend(old.into_iter().zip(new.into_iter()));
            }
            _ => {
                unreachable!(
                    "The inner operations are identical, so their updates should be the same variant."
                );
            }
        };
        match process_sync_inner(
            &problem,
            known_in_progress.ops[blocks + 1..].iter(),
            time_update,
        ) {
            SyncInnerStatus::RollbackFound => {
                SyncStatus::SyncError(Box::new(SyncError::RollbackFound(known_in_progress)))
            }
            SyncInnerStatus::Blockage(blocks) => {
                self.blocks = blocks;
                SyncStatus::InProgress(Box::new(Self {
                    known,
                    known_in_progress,
                    accepted,
                    other,
                    problem,
                    blocks,
                }))
            }
            SyncInnerStatus::Passes(None) => {
                accepted.add_op(problem);
                process_sync(known, known_in_progress, other, accepted)
            }
            SyncInnerStatus::Passes(Some(i)) => {
                let len = min(known_in_progress.len() - 1, i);
                known_in_progress.ops.drain(0..=len);
                other.ops.iter_mut().for_each(|op| {
                    player_updates
                        .iter()
                        .for_each(|(old, new)| op.swap_player_ids(*old, *new));
                    round_updates
                        .iter()
                        .for_each(|(old, new)| op.swap_round_ids(*old, *new));
                });
                process_sync(known, known_in_progress, other, accepted)
            }
        }
    }
}

pub(crate) fn process_sync(
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
        match process_sync_inner(&op, known.ops.iter(), time_update) {
            SyncInnerStatus::Passes(Some(i)) => {
                let len = min(known_in_progress.len() - 1, i);
                known_in_progress.ops.drain(0..=len);
            }
            SyncInnerStatus::Passes(None) => {
                accepted.ops.push(op);
            }
            SyncInnerStatus::Blockage(blocks) => {
                return SyncStatus::InProgress(Box::new(Blockage {
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

enum SyncInnerStatus {
    Passes(Option<usize>),
    Blockage(usize),
    RollbackFound,
}

fn process_sync_inner<'a, I, F>(other: &'a FullOp, known: I, mut time_update: F) -> SyncInnerStatus
where
    I: Iterator<Item = &'a FullOp>,
    F: FnMut(OpUpdate, OpUpdate),
{
    for (i, known_op) in known.enumerate() {
        match other.diff(known_op) {
            OpDiff::Different => {
                // TODO: Verify that these are ordered correctly
                if other.blocks(known_op) {
                    return SyncInnerStatus::Blockage(i);
                }
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
