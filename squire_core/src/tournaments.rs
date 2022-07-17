use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{get, post, serde::json::Json};

use squire_lib::tournament::{Tournament, TournamentId, TournamentIdentifier};
use squire_sdk::tournaments::{
    self, ApplyOpRequest, ApplyOpResponse, CreateResponse, GetAllResponse, GetResponse,
    StandingsRequest, StandingsResponse, TournamentCreateRequest, TournamentGetRequest,
};

pub static TOURNS_MAP: OnceCell<DashMap<TournamentId, Tournament>> = OnceCell::new();

#[post("/create", format = "json", data = "<data>")]
pub fn create_tournament(data: Json<TournamentCreateRequest>) -> CreateResponse {
    let tourn = Tournament::from_preset(data.0.name, data.0.preset, data.0.format);
    let id = tourn.id.clone();
    TOURNS_MAP.get().unwrap().insert(id, tourn.clone());
    tournaments::CreateResponse::new(tourn)
}

#[get("/get", format = "json", data = "<data>")]
pub fn get_tournament(data: Json<TournamentGetRequest>) -> GetResponse {
    match data.0.ident {
        TournamentIdentifier::Id(id) => {
            tournaments::GetResponse::new(TOURNS_MAP.get().unwrap().get(&id).map(|a| a.clone()))
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/all")]
pub fn get_all_tournaments() -> GetAllResponse {
    let map = TOURNS_MAP
        .get()
        .unwrap()
        .iter()
        .map(|r| (r.key().clone(), r.value().clone()))
        .collect();
    GetAllResponse::new(map)
}

#[get("/standings", format = "json", data = "<data>")]
pub fn get_standings(data: Json<StandingsRequest>) -> StandingsResponse {
    match data.0.ident {
        TournamentIdentifier::Id(id) => StandingsResponse::new(
            TOURNS_MAP
                .get()
                .unwrap()
                .get(&id)
                .map(|a| a.get_standings()),
        ),
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[post("/manage/list_ops", format = "json", data = "<data>")]
pub fn list_ops(data: Json<ListOpsRequest>) -> ListOpsResponse {
    match data.0.ident {
        TournamentIdentifier::Id(id) => {
            todo!()
        }
        TournamentIdentifier::Name(_name) => {
            todo!()
        }
    }
}

#[post("/manage/sync", format = "json", data = "<data>")]
pub fn list_ops(data: Json<SyncRequest>) -> SyncResponse {
    match data.0.ident {
        TournamentIdentifier::Id(id) => {
            todo!()
        }
        TournamentIdentifier::Name(_name) => {
            todo!()
        }
    }
}

#[post("/manage/rollback", format = "json", data = "<data>")]
pub fn list_ops(data: Json<RollbackRequest>) -> RollbackResponse {
    match data.0.ident {
        TournamentIdentifier::Id(id) => {
            todo!()
        }
        TournamentIdentifier::Name(_name) => {
            todo!()
        }
    }
}
