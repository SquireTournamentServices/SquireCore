use serde::{Deserialize, Serialize};

use crate::sync::{op_log::OpSlice, op_sync::OpSync, sync_error::SyncError};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A struct used to communicate a rollback
pub struct Rollback {
    pub(crate) ops: OpSlice,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// An enum that encodes that errors that can occur during a rollback
pub enum RollbackError {
    /// The rollback slice has an unknown starting point
    SliceError(SyncError),
    /// The log that doesn't contain the rollback contains operations that the rolled back log
    /// doesn't contain
    OutOfSync(OpSync),
}
