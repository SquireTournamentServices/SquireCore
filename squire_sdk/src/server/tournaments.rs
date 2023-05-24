use std::sync::Arc;

use async_session::SessionStore;
use axum::{
    extract::{Path, State},
    handler::Handler,
    routing::{get, post},
    Json, Router,
};

use crate::{
    api::{
        CREATE_TOURNAMENT_ENDPOINT, GET_ALL_ACTIVE_TOURNAMENTS_ENDPOINT, GET_TOURNAMENT_ENDPOINT,
        ROLLBACK_TOURNAMENT_ENDPOINT, SYNC_TOURNAMENT_ENDPOINT,
    },
    server::{state::ServerState, User},
    tournaments::*,
    utils::Url,
};

pub fn get_routes<S>() -> Router<S>
where
    S: ServerState,
{
    Router::new()
        .route(GET_TOURNAMENT_ENDPOINT.as_str(), get(get_tournament::<S>))
        .route(
            GET_ALL_ACTIVE_TOURNAMENTS_ENDPOINT.as_str(),
            get(get_all_tournaments::<S>),
        )
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

pub async fn get_all_tournaments<S>(State(state): State<S>) -> GetAllTournamentsResponse
where
    S: ServerState,
{
    GetAllTournamentsResponse::new(state.query_all_tournaments(|t| t.clone()).await)
}

/// Adds a user to the gathering via a subsocket
pub async fn join_gathering() {
    todo!()
}
