use serde::{Deserialize, Serialize};
use squire_lib::{error::TournamentError, tournament::Tournament};

use super::{FullOp, OpLog, OpSlice, OpSync, SyncError, OpId};

/// This type results from a client making a decision about what operations need to stay and what
/// operations need to be removed from its log during the sync process.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SyncDecision {
    /// The client decided to ignore the problematic operation and is asking the backend to
    /// continue trying to process things.
    Plucked(SyncProcessor),
    /// The client decided to ignore the rest of its log. This will mark the processing as done.
    Purged(SyncCompletion),
}

/// This type encodes the result of a successful sync.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SyncCompletion {
    /// The backend did not have any operations that were unknown to the client.
    ForeignOnly(OpSlice),
    /// The backend had one or more operations that were unknown to the client and the logs were
    /// successfully merged.
    Mixed(OpSlice),
}

/// This struct contain an in-progress sync. The processor is mostly used internally by the
/// `TournamentManager` to process sync requests; however, it is also shared during the sync
/// process so that the client can audit its log. Those methods produce an `OpDecision`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SyncProcessor {
    /// Whether or not sync contains only foreign operations
    foreign_only: bool,
    last_known_op: Option<OpId>,
    agreed: OpSlice,
    to_process: OpSlice,
}

impl SyncProcessor {
    /// This method is called by the client to decided how to process errors in the sync process.
    /// This method simply removes the operation that is causing the problem, preserving both the
    /// rest of its log
    pub fn pluck(mut self) -> SyncDecision {
        self.to_process.pop_front();
        SyncDecision::Plucked(self)
    }

    /// This method is called by the client to decided how to process errors in the sync process.
    /// This method removes all of the remaining operations that it is trying to sync.
    pub fn purge(self) -> SyncDecision {
        SyncDecision::Purged(self.finalize())
    }

    /// Returns whether or not the process will merge two logs or one
    pub fn is_foreign_only(&self) -> bool {
        self.foreign_only
    }

    /// Creates a new processor from an `OpSync` and an `OpLog`. This method is fallible for a
    /// number of reasons, the sync could be a mismatch with the log, the sync could be empty, and
    /// the sync might have an incorrect anchor operation.
    pub(crate) fn new(sync: OpSync, log: &OpLog) -> Result<Self, SyncError> {
        sync.validate(log)?;
        let id = sync.first_id()?;
        let known = match log.get_slice(id) {
            Some(slice) => slice,
            None if log.is_empty() => OpSlice::new(),
            _ => return Err(SyncError::UnknownOperation(id)),
        };
        let foreign_only = known.len() >= 1;
        Ok(Self {
            foreign_only,
            last_known_op: known.last_id(),
            agreed: known,
            to_process: sync.ops,
        })
    }

    pub(crate) fn last_known(&self) -> Option<OpId> {
        self.last_known_op
    }

    pub(crate) fn next_op(&self) -> Option<FullOp> {
        self.to_process.first_op()
    }

    pub(crate) fn move_op(&mut self) {
        let Some(op) = self.to_process.pop_front() else { return };
        self.agreed.add_op(op);
    }

    pub(crate) fn move_all(&mut self) {
        self.agreed.extend(self.to_process.drain())
    }

    pub(crate) fn finalize(self) -> SyncCompletion {
        if self.is_foreign_only() {
            SyncCompletion::ForeignOnly(self.agreed)
        } else {
            SyncCompletion::Mixed(self.agreed)
        }
    }
}

fn apply_ops<I: Iterator<Item = FullOp>>(
    tourn: &mut Tournament,
    ops: I,
) -> Result<(), TournamentError> {
    for op in ops {
        tourn.apply_op(op.salt, op.op)?;
    }
    Ok(())
}
