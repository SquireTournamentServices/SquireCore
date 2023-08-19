use axum::{extract::State, Json};
use http::StatusCode;
use squire_sdk::{
    api::*,
    model::{accounts::SquireAccount, identifiers::SquireAccountId},
    server::{
        session::{Session, SessionConvert, SessionToken, SquireSession, TokenParseError},
        state::ServerState,
    },
};

use crate::state::AppState;

pub async fn create_account(
    State(state): State<AppState>,
    Json(form): Json<RegForm>,
) -> (SessionToken, Json<SquireAccountId>) {
    let id = state.create_account(form).await;
    let session = state.create_session(id).await;
    (session, Json(id))
}

pub struct ActiveSession(SquireAccountId);

impl SessionConvert for ActiveSession {
    type Error = StatusCode;

    fn convert(_token: SessionToken, session: SquireSession) -> Result<Self, Self::Error> {
        if let SquireSession::Active(id) = session {
            Ok(Self(id))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }

    fn empty_session(_err: TokenParseError) -> Result<Self, Self::Error> {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn get_account(
    State(state): State<AppState>,
    Session(ActiveSession(id)): Session<ActiveSession>,
) -> Json<Option<SquireAccount>> {
    Json(state.get_account(id).await)
}

pub async fn delete_account(
    State(state): State<AppState>,
    Session(ActiveSession(id)): Session<ActiveSession>,
) -> StatusCode {
    if state.delete_account(id).await {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}
