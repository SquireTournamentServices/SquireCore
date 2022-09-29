use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{get, post, serde::json::Json};

use squire_lib::tournament::{Tournament, TournamentId};
use squire_sdk::tournaments::{
    self, AllTournamentsResponse, CreateTournamentRequest, CreateTournamentResponse,
    GetTournamentResponse, OpSliceResponse, RollbackRequest, RollbackResponse, StandingsResponse,
    SyncRequest, SyncResponse,
};
use uuid::Uuid;

pub static TOURNS_MAP: OnceCell<DashMap<TournamentId, Tournament>> = OnceCell::new();

#[post("/create", format = "json", data = "<data>")]
pub fn create_tournament(data: Json<CreateTournamentRequest>) -> CreateTournamentResponse {
    let tourn = Tournament::from_preset(data.0.name, data.0.preset, data.0.format);
    let id = tourn.id.clone();
    TOURNS_MAP.get().unwrap().insert(id, tourn.clone());
    CreateTournamentResponse::new(tourn)
}

#[get("/<id>/get")]
pub fn get_tournament(id: Uuid) -> GetTournamentResponse {
    GetTournamentResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(id))
            .map(|a| a.clone()),
    )
}

#[get("/all")]
pub fn get_all_tournaments() -> AllTournamentsResponse {
    let map = TOURNS_MAP
        .get()
        .unwrap()
        .iter()
        .map(|r| (r.key().clone(), r.value().clone()))
        .collect();
    AllTournamentsResponse::new(map)
}

#[get("/<id>/standings")]
pub fn get_standings(id: Uuid) -> StandingsResponse {
    StandingsResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(id))
            .map(|a| a.get_standings()),
    )
}

#[get("/<t_id>/manage/ops/slice/<o_id>")]
pub fn slice_ops(t_id: Uuid, o_id: Uuid) -> OpSliceResponse {
    OpSliceResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(t_id))
            .map(|a| todo!()),
    )
}

#[post("/<id>/manage/refresh", format = "json", data = "<data>")]
pub fn refresh(id: Uuid, data: Json<SyncRequest>) -> SyncResponse {
    SyncResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(id))
            .map(|a| todo!()),
    )
}

#[post("/<id>/manage/sync", format = "json", data = "<data>")]
pub fn sync(id: Uuid, data: Json<SyncRequest>) -> SyncResponse {
    SyncResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(id))
            .map(|a| todo!()),
    )
}

#[post("/<id>/manage/rollback", format = "json", data = "<data>")]
pub fn rollback(id: Uuid, data: Json<RollbackRequest>) -> RollbackResponse {
    RollbackResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(id))
            .map(|a| todo!()),
    )
}
