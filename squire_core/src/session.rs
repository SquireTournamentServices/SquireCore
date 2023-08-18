use axum::{extract::State, Json};
use http::StatusCode;
use squire_sdk::{
    api::{Login, SessionToken},
    model::accounts::SquireAccount,
    server::{
        session::{AnyUser, Session},
        state::ServerState,
    },
};

use crate::state::{AppState, LoginError};

/// Takes user credentials (username and password) and returns a new session token to them
/// (provided the credentials match known credentials).
pub async fn login(
    State(state): State<AppState>,
    Json(Login(cred)): Json<Login>,
) -> Result<(SessionToken, Json<SquireAccount>), StatusCode> {
    let token = state
        .login(cred)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    state
        .get_account_by_session(token.clone())
        .await
        .map(|acc| (token, Json(acc)))
        .ok_or(StatusCode::BAD_REQUEST)
}

/// Generates a guest session
pub async fn guest(State(state): State<AppState>) -> SessionToken {
    state.guest_session().await
}

/// Reauthenticates a user by issuing a new session token to them. The user must either have an
/// active session or a recently expired session. Otherwise, they need to go through `login`.
pub async fn reauth(
    State(state): State<AppState>,
    Session(session): Session<AnyUser>,
) -> SessionToken {
    state.reauth_session(session).await
}

/// Terminates a session.
pub async fn terminate(
    State(state): State<AppState>,
    Session(session): Session<AnyUser>,
) -> Json<bool> {
    Json(state.terminate_session(session).await)
}
