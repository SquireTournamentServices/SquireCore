use std::collections::VecDeque;

use squire_lib::{accounts::SquireAccount, error::TournamentError, tournament::TournamentSeed};

use crate::sync::{FullOp, OpSlice, OpSync};

use super::{
    processor::{SyncCompletion, SyncProcessor},
    Disagreement, ForwardError, OpId, RequestError, ServerOpLink, SyncError, SyncForwardResp,
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

impl From<TournamentError> for SyncError {
    fn from(value: TournamentError) -> Self {
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

impl From<TournamentError> for RequestError {
    fn from(value: TournamentError) -> Self {
        Self::TournError(value)
    }
}

/* ---- SyncForwardResp Helper Traits ---- */
impl From<ForwardError> for SyncForwardResp {
    fn from(value: ForwardError) -> Self {
        SyncForwardResp::Error(value)
    }
}

impl From<RequestError> for SyncForwardResp {
    fn from(value: RequestError) -> Self {
        SyncForwardResp::Error(value.into())
    }
}

impl From<TournamentError> for SyncForwardResp {
    fn from(value: TournamentError) -> Self {
        SyncForwardResp::Error(value.into())
    }
}

/* ---- ForwardError Helper Traits ---- */
impl From<RequestError> for ForwardError {
    fn from(value: RequestError) -> Self {
        ForwardError::InvalidRequest(value)
    }
}

impl From<TournamentError> for ForwardError {
    fn from(value: TournamentError) -> Self {
        ForwardError::TournError(value)
    }
}
