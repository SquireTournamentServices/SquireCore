use serde::{Deserialize, Serialize};

use crate::model::{accounts::SquireAccount, tournament::TournamentSeed};

use super::{op_log::OpSlice, sync_error::SyncError, FullOp, OpId, OpLog};

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
            Err(SyncError::UnknownOperation(self.first_id()?))
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
