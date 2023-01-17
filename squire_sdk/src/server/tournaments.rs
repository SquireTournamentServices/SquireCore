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
        CREATE_TOURNAMENT_ENDPOINT, GET_TOURNAMENT_ENDPOINT, ROLLBACK_TOURNAMENT_ENDPOINT,
        SYNC_TOURNAMENT_ENDPOINT,
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
        .route(
            CREATE_TOURNAMENT_ENDPOINT.as_str(),
            post(create_tournament::<S>),
        )
        .route(GET_TOURNAMENT_ENDPOINT.as_str(), get(get_tournament::<S>))
        .route(SYNC_TOURNAMENT_ENDPOINT.as_str(), post(sync::<S>))
        .route(ROLLBACK_TOURNAMENT_ENDPOINT.as_str(), post(rollback::<S>))
}

pub async fn create_tournament<S>(
    user: User,
    State(state): State<S>,
    Json(data): Json<CreateTournamentRequest>,
) -> CreateTournamentResponse
where
    S: ServerState,
{
    CreateTournamentResponse::new(
        state
            .create_tournament(user, data.name, data.preset, data.format)
            .await,
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

pub async fn sync<S>(
    user: User,
    State(state): State<S>,
    id: Path<TournamentId>,
    data: Json<SyncRequest>,
) -> SyncResponse
where
    S: ServerState,
{
    SyncResponse::new(state.sync_tournament(&id, &user, data.0.sync).await)
}

pub async fn rollback<S>(
    user: User,
    State(state): State<S>,
    id: Path<TournamentId>,
    data: Json<RollbackRequest>,
) -> RollbackResponse
where
    S: ServerState,
{
    RollbackResponse::new(state.rollback_tournament(&id, &user, data.0.rollback).await)
}
