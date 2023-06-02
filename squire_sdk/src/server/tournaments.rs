use std::sync::Arc;

use async_session::SessionStore;
use axum::{
    extract::{Path, State},
    handler::Handler,
    routing::{get, post},
    Json, Router,
};

use crate::{
    api::GET_TOURNAMENT_ENDPOINT,
    server::{state::ServerState, User},
    tournaments::*,
    utils::Url,
};

pub fn get_routes<S>() -> Router<S>
where
    S: ServerState,
{
    Router::new().route(GET_TOURNAMENT_ENDPOINT.as_str(), get(get_tournament::<S>))
}

pub async fn get_tournament<S>(
    State(state): State<S>,
    id: Path<TournamentId>,
) -> GetTournamentResponse
where
    S: ServerState,
{
    GetTournamentResponse::new(state.query_tournament(&id, |t| t.clone()).await)
}

/// Adds a user to the gathering via a websocket
pub async fn join_gathering() {
    todo!()
}
