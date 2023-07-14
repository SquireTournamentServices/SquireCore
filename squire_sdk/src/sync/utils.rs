use std::{collections::VecDeque, convert::Infallible, ops::FromResidual};

use squire_lib::{
    accounts::SquireAccount,
    error::TournamentError,
    tournament::tournament::TournamentId,
};
use squire_lib::tournament::tournament_seed::TournamentSeed;

use super::{
    ClientBound,
    ClientOpLink, Disagreement, ForwardError, processor::{SyncCompletion, SyncDecision, SyncProcessor}, RequestError, ServerBound, ServerOpLink,
    SyncError, SyncForwardResp, TournamentManager,
};
use crate::sync::{FullOp, OpSlice, OpSync};

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

impl From<(TournamentId, OpSync)> for ClientBound {
    fn from(value: (TournamentId, OpSync)) -> Self {
        Self::SyncForward(value)
    }
}

impl From<TournamentManager> for ClientBound {
    fn from(value: TournamentManager) -> Self {
        Self::FetchResp(Box::new(value))
    }
}

/* ---- SyncError Helper Traits ---- */
impl From<RequestError> for SyncError {
    fn from(value: RequestError) -> Self {
        Self::InvalidRequest(Box::new(value))
    }
}

impl From<Disagreement<SquireAccount>> for SyncError {
    fn from(value: Disagreement<SquireAccount>) -> Self {
        Self::InvalidRequest(Box::new(value.into()))
    }
}

impl From<Disagreement<TournamentSeed>> for SyncError {
    fn from(value: Disagreement<TournamentSeed>) -> Self {
        Self::InvalidRequest(Box::new(value.into()))
    }
}

impl From<TournamentError> for SyncError {
    fn from(value: TournamentError) -> Self {
        Self::InvalidRequest(Box::new(value.into()))
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
        SyncForwardResp::Error(Box::new(value))
    }
}

impl From<RequestError> for SyncForwardResp {
    fn from(value: RequestError) -> Self {
        SyncForwardResp::Error(Box::new(value.into()))
    }
}

impl From<TournamentError> for SyncForwardResp {
    fn from(value: TournamentError) -> Self {
        SyncForwardResp::Error(Box::new(value.into()))
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
        ForwardError::TournError(Box::new(value))
    }
}

/* ---- ClientOpLink Helper Traits ---- */

impl From<OpSync> for ClientOpLink {
    fn from(value: OpSync) -> Self {
        Self::Init(value)
    }
}

impl From<SyncDecision> for ClientOpLink {
    fn from(value: SyncDecision) -> Self {
        Self::Decision(value)
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

impl From<RequestError> for ServerOpLink {
    fn from(value: RequestError) -> Self {
        Self::Error(value.into())
    }
}

impl FromResidual<Result<Infallible, SyncError>> for ServerOpLink {
    fn from_residual(residual: Result<Infallible, SyncError>) -> Self {
        Self::Error(residual.unwrap_err())
    }
}
