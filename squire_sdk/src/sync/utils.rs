use std::collections::VecDeque;

use squire_lib::{accounts::SquireAccount, tournament::TournamentSeed};

use crate::sync::{FullOp, OpSlice, OpSync};

use super::{
    processor::{SyncCompletion, SyncProcessor},
    Disagreement, OpId, RequestError, ServerOpLink, SyncError,
};

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

impl From<OpSync> for OpSlice {
    fn from(s: OpSync) -> OpSlice {
        s.ops
    }
}

/* ---- ServerOpLink Helper Traits ---- */

impl From<SyncProcessor> for ServerOpLink {
    fn from(value: SyncProcessor) -> Self {
        Self::Conflict(value)
    }
}

impl From<SyncCompletion> for ServerOpLink {
    fn from(value: SyncCompletion) -> Self {
        Self::Completed(value)
    }
}

impl From<bool> for ServerOpLink {
    fn from(already_done: bool) -> Self {
        Self::TerminatedSeen { already_done }
    }
}

impl From<SyncError> for ServerOpLink {
    fn from(value: SyncError) -> Self {
        Self::Error(value)
    }
}

/* ---- SyncError Helper Traits ---- */
impl From<RequestError> for SyncError {
    fn from(value: RequestError) -> Self {
        Self::InvalidRequest(value)
    }
}

impl From<Disagreement<SquireAccount>> for SyncError {
    fn from(value: Disagreement<SquireAccount>) -> Self {
        Self::InvalidRequest(value.into())
    }
}

impl From<Disagreement<TournamentSeed>> for SyncError {
    fn from(value: Disagreement<TournamentSeed>) -> Self {
        Self::InvalidRequest(value.into())
    }
}

impl From<OpId> for SyncError {
    fn from(value: OpId) -> Self {
        Self::InvalidRequest(value.into())
    }
}

/* ---- RequestError Helper Traits ---- */
impl From<Disagreement<TournamentSeed>> for RequestError {
    fn from(value: Disagreement<TournamentSeed>) -> Self {
        Self::WrongSeed(value)
    }
}

impl From<Disagreement<SquireAccount>> for RequestError {
    fn from(value: Disagreement<SquireAccount>) -> Self {
        Self::WrongAccount(value)
    }
}

impl From<OpId> for RequestError {
    fn from(value: OpId) -> Self {
        Self::OpCountIncreased(value)
    }
}
