use async_session::MemoryStore;
use axum::{extract::{Path, State, FromRef}, Json};
use dashmap::DashMap;
use once_cell::sync::OnceCell;

use squire_sdk::model::{
    identifiers::OpId,
    tournament::{Tournament, TournamentId},
    tournament_manager::TournamentManager,
};
use squire_sdk::tournaments::{
    self, AllTournamentsResponse, CreateTournamentRequest, CreateTournamentResponse,
    GetTournamentResponse, OpSliceResponse, RollbackRequest, RollbackResponse, StandingsResponse,
    SyncRequest, SyncResponse,
};

use crate::User;

pub static TOURNS_MAP: OnceCell<DashMap<TournamentId, TournamentManager>> = OnceCell::new();


pub fn init() {
    TOURNS_MAP.get_or_init(Default::default);
}

pub async fn create_tournament(user: User, Json(data): Json<CreateTournamentRequest>) -> CreateTournamentResponse {
    let tourn = user.account.create_tournament(data.name, data.preset, data.format);
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
