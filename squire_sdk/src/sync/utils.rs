use std::{ops::Deref, collections::VecDeque};

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

impl From<VecDeque<FullOp>> for OpSlice {
    fn from(ops: VecDeque<FullOp>) -> Self {
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
