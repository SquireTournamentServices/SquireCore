use std::sync::Arc;

use async_session::SessionStore;
use axum::{
    extract::{Path, State},
    handler::Handler,
    routing::{get, post},
    Json, Router,
};

use crate::{
    server::{state::ServerState, User},
    tournaments::*,
};

pub fn get_routes<S>() -> Router<S>
where
    S: ServerState,
{
    Router::new()
        .route("/create", post(create_tournament::<S>))
        .route("/:t_id", get(get_tournament::<S>))
        .route("/:t_id/sync", post(sync::<S>))
        .route("/:t_id/rollback", post(rollback::<S>))
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
