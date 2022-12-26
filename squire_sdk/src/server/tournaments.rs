use async_session::SessionStore;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};

use crate::{
    server::{AppState, User},
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
    Json(data): Json<CreateTournamentRequest>,
) -> CreateTournamentResponse {
    let tourn = user
        .account
        .create_tournament(data.name, data.preset, data.format);
    let id = tourn.id;
    TOURNS_MAP.get().unwrap().insert(id, tourn.clone());
    CreateTournamentResponse::new(tourn)
}

pub async fn get_tournament(id: Path<TournamentId>) -> GetTournamentResponse {
    GetTournamentResponse::new(TOURNS_MAP.get().unwrap().get(&id).map(|a| a.clone()))
}

pub async fn sync(id: Path<TournamentId>, data: Json<SyncRequest>) -> SyncResponse {
    SyncResponse::new(TOURNS_MAP.get().unwrap().get(&id).map(|a| todo!()))
}

pub async fn rollback(id: Path<TournamentId>, data: Json<RollbackRequest>) -> RollbackResponse {
    RollbackResponse::new(TOURNS_MAP.get().unwrap().get(&id).map(|a| todo!()))
}
