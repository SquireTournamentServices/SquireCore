use rocket::{get, request::FromParam, serde::json::Json};

use squire_lib::{
    identifiers::{PlayerId, PlayerIdentifier, TypeId},
    tournament::{TournamentId, TournamentIdentifier},
};
use squire_sdk::players::{
    GetAllDecksResponse, GetAllPlayerDecksResponse, GetDeckResponse, GetLatestPlayerMatchResponse,
    GetMultiplePlayersResponse, GetPlayerCountResponse, GetPlayerMatchesResponse,
    GetPlayerResponse,
};
use uuid::Uuid;

use crate::tournaments::TOURNS_MAP;

#[get("/<t_id>/players/<p_id>/get")]
pub fn get_player(t_id: Uuid, p_id: Uuid) -> GetPlayerResponse {
    GetPlayerResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(t_id))
            .map(|tourn| {
                tourn
                    .get_player(&PlayerIdentifier::Id(PlayerId::new(p_id)))
                    .map(|p| p.clone())
                    .ok()
            }),
    )
}

#[get("/<t_id>/players/get/all")]
pub fn get_all_players(t_id: Uuid) -> GetMultiplePlayersResponse {
    GetMultiplePlayersResponse::new(TOURNS_MAP.get().unwrap().get(&TournamentId::new(t_id)).map(
        |tourn| {
            tourn
                .player_reg
                .players
                .iter()
                .map(|(id, p)| (id.clone(), p.clone()))
                .collect()
        },
    ))
}

#[get("/<t_id>/players/get/active")]
pub fn get_active_players(t_id: Uuid) -> GetMultiplePlayersResponse {
    GetMultiplePlayersResponse::new(TOURNS_MAP.get().unwrap().get(&TournamentId::new(t_id)).map(
        |tourn| {
            tourn
                .player_reg
                .players
                .iter()
                .filter(|(_, p)| p.can_play())
                .map(|(id, p)| (id.clone(), p.clone()))
                .collect()
        },
    ))
}

#[get("/<t_id>/players/count")]
pub fn get_player_count(t_id: Uuid) -> GetPlayerCountResponse {
    GetPlayerCountResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(t_id))
            .map(|tourn| tourn.player_reg.players.len() as u64),
    )
}

#[get("/<t_id>/players/active_count")]
pub fn get_active_player_count(t_id: Uuid) -> GetPlayerCountResponse {
    GetPlayerCountResponse::new(
        TOURNS_MAP
            .get()
            .unwrap()
            .get(&TournamentId::new(t_id))
            .map(|tourn| tourn.player_reg.active_player_count() as u64),
    )
}

#[get("/<t_id>/players/<p_id>/matches")]
pub fn get_player_matches(t_id: Uuid, p_id: Uuid) -> GetPlayerMatchesResponse {
    let p_id = PlayerIdentifier::Id(PlayerId::new(p_id));
    GetPlayerMatchesResponse::new(TOURNS_MAP.get().unwrap().get(&TournamentId::new(t_id)).map(
        |tourn| {
            tourn.get_player(&p_id).ok().map(|p| {
                tourn
                    .round_reg
                    .rounds
                    .iter()
                    .filter(|(_, r)| r.players.contains(&p.id))
                    .map(|(_, r)| r.clone())
                    .collect()
            })
        },
    ))
}

#[get("/<t_id>/players/<p_id>/latest_match")]
pub fn get_latest_player_match(t_id: Uuid, p_id: Uuid) -> GetLatestPlayerMatchResponse {
    let p_id = PlayerIdentifier::Id(PlayerId::new(p_id));
    // TODO: This is techincally incorrect. Fix
    GetLatestPlayerMatchResponse::new(TOURNS_MAP.get().unwrap().get(&TournamentId::new(t_id)).map(
        |tourn| {
            tourn.get_player(&p_id).ok().map(|p| {
                tourn
                    .round_reg
                    .rounds
                    .iter()
                    .find(|(_, r)| r.players.contains(&p.id))
                    .map(|(_, r)| r.clone())
            })
        },
    ))
}

#[get("/<t_id>/players/<p_id>/decks/get/<name>")]
pub fn get_player_deck(t_id: Uuid, p_id: Uuid, name: String) -> GetDeckResponse {
    let p_id: PlayerId = p_id.into();
    GetDeckResponse::new(TOURNS_MAP.get().unwrap().get(&(t_id.into())).map(|tourn| {
        tourn
            .get_player(&(p_id.into()))
            .ok()
            .map(|p| p.get_deck(&name))
    }))
}

#[get("/<t_id>/players/<p_id>/decks/all")]
pub fn get_all_player_decks(t_id: Uuid, p_id: Uuid) -> GetAllPlayerDecksResponse {
    let p_id: PlayerId = p_id.into();
    GetAllPlayerDecksResponse::new(TOURNS_MAP.get().unwrap().get(&(t_id.into())).map(|tourn| {
        tourn
            .get_player(&(p_id.into()))
            .ok()
            .map(|p| p.decks.clone())
    }))
}

#[get("/<t_id>/decks/all")]
pub fn get_all_decks(t_id: Uuid) -> GetAllDecksResponse {
    GetAllDecksResponse::new(TOURNS_MAP.get().unwrap().get(&(t_id.into())).map(|tourn| {
        tourn
            .player_reg
            .players
            .iter()
            .map(|(id, p)| (id.clone(), p.decks.clone()))
            .collect()
    }))
}
