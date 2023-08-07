use axum::{extract::State, response::IntoResponse};
use http::StatusCode;
use squire_sdk::{
    model::identifiers::SquireAccountId,
    server::session::{Session, SessionConvert, SquireSession},
};

use crate::state::AppState;

/// Takes user credentials (username and password) and returns a new session token to them
/// (provided the credentials match known credentials).
pub async fn login(State(_state): State<AppState>) -> impl IntoResponse {
    todo!()
}

pub enum ReauthSession {
    Active(SquireAccountId),
    Expired(SquireAccountId),
}

impl SessionConvert for ReauthSession {
    type Error = StatusCode;

    fn convert(session: SquireSession) -> Result<Self, Self::Error> {
        match session {
            SquireSession::Active(id) => Ok(Self::Active(id)),
            SquireSession::Expired(id) => Ok(Self::Expired(id)),
            SquireSession::NotLoggedIn | SquireSession::UnknownUser => {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

/// Reauthenticates a user by issuing a new session token to them. The user must either have an
/// active session or a recently expired session. Otherwise, they need to go through `login`.
pub async fn reauth(
    State(_state): State<AppState>,
    Session(_session): Session<ReauthSession>,
) -> impl IntoResponse {
    todo!()
}

/// Terminates a session.
pub async fn terminate(
    State(_state): State<AppState>,
    Session(_session): Session<ReauthSession>,
) -> impl IntoResponse {
    todo!()
}
