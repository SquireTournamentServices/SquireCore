use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{get, post, serde::json::Json};

use squire_lib::tournament::{Tournament, TournamentId, TournamentIdentifier};
use squire_sdk::players::{
    GetAllDecksRequest, GetAllDecksResponse, GetAllPlayerDecksRequest, GetAllPlayerDecksResponse,
    GetDeckRequest, GetDeckResponse, GetLatestPlayerMatchRequest, GetLatestPlayerMatchResponse,
    GetMultiplePlayersRequest, GetMultiplePlayersResponse, GetPlayerCountRequest,
    GetPlayerCountResponse, GetPlayerMatchesRequest, GetPlayerMatchesResponse, GetPlayerRequest,
    GetPlayerResponse,
};

use crate::tournaments::TOURNS_MAP;

#[get("/get", format = "json", data = "<data>")]
pub fn get_player(data: Json<GetPlayerRequest>) -> GetPlayerResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP
                .get()
                .unwrap()
                .get(&id)
                .map(|tourn| tourn.get_player(&data.0.player).map(|p| p.clone()).ok());
            GetPlayerResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/all", format = "json", data = "<data>")]
pub fn get_all_players(data: Json<GetMultiplePlayersRequest>) -> GetMultiplePlayersResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP.get().unwrap().get(&id).map(|tourn| {
                tourn
                    .player_reg
                    .players
                    .iter()
                    .map(|(id, p)| (id.clone(), p.clone()))
                    .collect()
            });
            GetMultiplePlayersResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/active", format = "json", data = "<data>")]
pub fn get_active_players(data: Json<GetMultiplePlayersRequest>) -> GetMultiplePlayersResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP.get().unwrap().get(&id).map(|tourn| {
                tourn
                    .player_reg
                    .players
                    .iter()
                    .filter(|(_, p)| p.can_play())
                    .map(|(id, p)| (id.clone(), p.clone()))
                    .collect()
            });
            GetMultiplePlayersResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/count", format = "json", data = "<data>")]
pub fn get_player_count(data: Json<GetPlayerCountRequest>) -> GetPlayerCountResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP
                .get()
                .unwrap()
                .get(&id)
                .map(|tourn| tourn.player_reg.players.len() as u64);
            GetPlayerCountResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/active_count", format = "json", data = "<data>")]
pub fn get_active_player_count(data: Json<GetPlayerCountRequest>) -> GetPlayerCountResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP
                .get()
                .unwrap()
                .get(&id)
                .map(|tourn| tourn.player_reg.active_player_count() as u64);
            GetPlayerCountResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/matches", format = "json", data = "<data>")]
pub fn get_player_matches(data: Json<GetPlayerMatchesRequest>) -> GetPlayerMatchesResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP.get().unwrap().get(&id).map(|tourn| {
                tourn.get_player(&data.0.player).ok().map(|p| {
                    tourn
                        .round_reg
                        .rounds
                        .iter()
                        .filter(|(_, r)| r.players.contains(&p.id))
                        .map(|(_, r)| r.clone())
                        .collect()
                })
            });
            GetPlayerMatchesResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/latest_match", format = "json", data = "<data>")]
pub fn get_latest_player_match(
    data: Json<GetLatestPlayerMatchRequest>,
) -> GetLatestPlayerMatchResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            // TODO: This is techincally incorrect. Fix
            let digest = TOURNS_MAP.get().unwrap().get(&id).map(|tourn| {
                tourn.get_player(&data.0.player).ok().map(|p| {
                    tourn
                        .round_reg
                        .rounds
                        .iter()
                        .find(|(_, r)| r.players.contains(&p.id))
                        .map(|(_, r)| r.clone())
                })
            });
            GetLatestPlayerMatchResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/decks/get", format = "json", data = "<data>")]
pub fn get_player_deck(data: Json<GetDeckRequest>) -> GetDeckResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP.get().unwrap().get(&id).map(|tourn| {
                tourn
                    .get_player(&data.0.player)
                    .ok()
                    .map(|p| p.get_deck(&data.0.deck_name))
            });
            GetDeckResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/decks/all", format = "json", data = "<data>")]
pub fn get_all_decks(data: Json<GetAllDecksRequest>) -> GetAllDecksResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP.get().unwrap().get(&id).map(|tourn| {
                tourn
                    .player_reg
                    .players
                    .iter()
                    .map(|(id, p)| (id.clone(), p.decks.clone()))
                    .collect()
            });
            GetAllDecksResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/decks/player_all", format = "json", data = "<data>")]
pub fn get_all_player_decks(data: Json<GetAllPlayerDecksRequest>) -> GetAllPlayerDecksResponse {
    match data.0.tourn {
        TournamentIdentifier::Id(id) => {
            let digest = TOURNS_MAP.get().unwrap().get(&id).map(|tourn| {
                tourn
                    .get_player(&data.0.player)
                    .ok()
                    .map(|p| p.decks.clone())
            });
            GetAllPlayerDecksResponse::new(digest)
        }
        TournamentIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}
