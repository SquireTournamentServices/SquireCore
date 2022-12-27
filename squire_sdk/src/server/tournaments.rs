use async_session::SessionStore;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::{
    server::{state::ServerState, AppState, User},
    tournaments::*,
};

pub fn get_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_tournament))
        .route("/:t_id", get(get_tournament))
        .route("/:t_id/sync", post(sync))
        .route("/:t_id/rollback", post(rollback))
}

pub async fn create_tournament(
    user: User,
    State(state): State<AppState>,
    Json(data): Json<CreateTournamentRequest>,
) -> CreateTournamentResponse {
    CreateTournamentResponse::new(
        state
            .create_tournament(user, data.name, data.preset, data.format)
            .await,
    )
}

pub async fn get_tournament(
    State(state): State<AppState>,
    id: Path<TournamentId>,
) -> GetTournamentResponse {
    GetTournamentResponse::new(state.query_tournament(&id, |t| t.clone()).await)
}

pub async fn sync(
    user: User,
    State(state): State<AppState>,
    id: Path<TournamentId>,
    data: Json<SyncRequest>,
) -> SyncResponse {
    SyncResponse::new(state.sync_tournament(&id, &user, data.0.sync).await)
}

pub async fn rollback(
    user: User,
    State(state): State<AppState>,
    id: Path<TournamentId>,
    data: Json<RollbackRequest>,
) -> RollbackResponse {
    RollbackResponse::new(state.rollback_tournament(&id, &user, data.0.rollback).await)
}
