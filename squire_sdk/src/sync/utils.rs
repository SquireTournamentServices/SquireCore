use std::collections::VecDeque;

use squire_lib::{accounts::SquireAccount, error::TournamentError, tournament::TournamentSeed};

use crate::sync::{FullOp, OpSlice, OpSync};

use super::{
    processor::{SyncCompletion, SyncProcessor},
    ClientBound, ClientOpLink, Disagreement, ForwardError, RequestError, ServerBound, ServerOpLink,
    SyncError, SyncForwardResp,
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

/* ---- ServerBound Helper Traits ---- */

impl From<ClientOpLink> for ServerBound {
    fn from(value: ClientOpLink) -> Self {
        Self::SyncChain(value)
    }
}

impl From<SyncForwardResp> for ServerBound {
    fn from(value: SyncForwardResp) -> Self {
        Self::ForwardResp(value)
    }
}

/* ---- ClientBound Helper Traits ---- */

impl From<ServerOpLink> for ClientBound {
    fn from(value: ServerOpLink) -> Self {
        Self::SyncChain(value)
    }
}

impl From<OpSync> for ClientBound {
    fn from(value: OpSync) -> Self {
        Self::SyncForward(value)
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
