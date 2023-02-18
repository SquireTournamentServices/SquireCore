use serde::{Deserialize, Serialize};

use crate::{
    model::error::TournamentError,
    sync::{op_log::OpSlice, op_sync::OpSync, FullOp}
};

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
    /// An error in the tournament occured (this should not happen)
    TournamentError(TournamentError),
}
