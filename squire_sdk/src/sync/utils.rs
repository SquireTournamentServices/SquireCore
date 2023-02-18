use std::ops::Deref;

use squire_lib::error::TournamentError;

use crate::{
    model::tournament::Tournament,
    sync::{
        full_op::FullOp, op_log::OpSlice, op_sync::OpSync, rollback::Rollback, TournamentManager,
    },
};

use super::SyncError;

impl Deref for TournamentManager {
    type Target = Tournament;

    fn deref(&self) -> &Self::Target {
        &self.tourn
    }
}

impl Default for OpSlice {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<FullOp>> for OpSlice {
    fn from(ops: Vec<FullOp>) -> Self {
        Self { ops }
    }
}

impl From<Rollback> for OpSlice {
    fn from(r: Rollback) -> OpSlice {
        r.ops
    }
}

impl From<OpSync> for OpSlice {
    fn from(s: OpSync) -> OpSlice {
        s.ops
    }
}

/* ---- SyncError Helper Traits ---- */
impl From<TournamentError> for SyncError {
    fn from(value: TournamentError) -> Self {
        SyncError::TournamentError(value)
    }
}

impl From<OpSlice> for SyncError {
    fn from(value: OpSlice) -> Self {
        SyncError::RollbackFound(value)
    }
}

impl From<(FullOp, OpSync)> for SyncError
{
    fn from(value: (FullOp, OpSync)) -> Self {
        SyncError::FailedSync(Box::new(value))
    }
}

impl From<Box<(FullOp, OpSync)>> for SyncError
{
    fn from(value: Box<(FullOp, OpSync)>) -> Self {
        SyncError::FailedSync(value)
    }
}

impl From<FullOp> for SyncError
{
    fn from(value: FullOp) -> Self {
        SyncError::UnknownOperation(Box::new(value))
    }
}

impl From<Box<FullOp>> for SyncError
{
    fn from(value: Box<FullOp>) -> Self {
        SyncError::UnknownOperation(value)
    }
}
