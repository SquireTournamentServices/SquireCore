use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use squire_sdk::{
    api::SessionToken,
    model::tournament::TournamentId,
    server::{
        session::{AnyUser, Session, SessionConvert},
        state::ServerState,
    },
};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionQuery {
    session: String,
}

/// Adds a user to the gathering via a websocket
pub async fn join_gathering(
    state: State<AppState>,
    ws: WebSocketUpgrade,
    path: Path<TournamentId>,
    Query(SessionQuery { session }): Query<SessionQuery>,
) -> Response {
    println!("Got alt join request...");
    match session.parse::<SessionToken>() {
        Ok(token) => match AnyUser::convert(token.clone(), state.0.get_session(token).await) {
            Ok(session) => {
                squire_sdk::server::tournaments::join_gathering(state, Session(session), ws, path)
                    .await
            }
            Err(_) => StatusCode::UNAUTHORIZED.into_response(),
        },
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}
