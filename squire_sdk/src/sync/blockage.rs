use std::cmp::min;

use serde::{Deserialize, Serialize};

use squire_lib::{tournament::Tournament, operations::OpUpdate};
use super::{
    process_sync, process_sync_inner, FullOp, OpSlice, OpSync, SyncError, SyncInnerStatus,
    SyncStatus,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A struct to help resolve blockages
pub struct Blockage {
    pub(crate) base: Tournament,
    pub(crate) known: OpSync,
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
            mut base,
            known,
            known_in_progress,
            accepted,
            other,
            ..
        } = self;
        process_sync(&mut base, known, known_in_progress, other, accepted)
    }

    /// Accepts that the problematic operation is meant to occur after the operation it blocks and
    /// continues the syncing process
    pub fn push(mut self) -> SyncStatus {
        let Self {
            base,
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
            &mut base,
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
                    base,
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
                process_sync(&mut base, known, known_in_progress, other, accepted)
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
                process_sync(&mut base, known, known_in_progress, other, accepted)
            }
        }
    }
}
