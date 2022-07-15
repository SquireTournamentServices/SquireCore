use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use mtgjson::model::deck::Deck;
pub use squire_lib::{
    error::TournamentError,
    player::{Player, PlayerId},
    player_registry::PlayerIdentifier,
    round::Round,
    tournament::TournamentIdentifier,
};

use crate::response::SquireResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPlayerRequest {
    pub tourn: TournamentIdentifier,
    pub player: PlayerIdentifier,
}

pub type GetPlayerResponse = SquireResponse<Option<Option<Player>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMultiplePlayersRequest {
    pub tourn: TournamentIdentifier,
}

pub type GetMultiplePlayersResponse = SquireResponse<Option<HashMap<PlayerId, Player>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDeckRequest {
    pub tourn: TournamentIdentifier,
    pub player: PlayerIdentifier,
    pub deck_name: String,
}

pub type GetDeckResponse = SquireResponse<Option<Option<Option<Deck>>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAllPlayerDecksRequest {
    pub tourn: TournamentIdentifier,
    pub player: PlayerIdentifier,
}

pub type GetAllPlayerDecksResponse = SquireResponse<Option<Option<HashMap<String, Deck>>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAllDecksRequest {
    pub tourn: TournamentIdentifier,
}

pub type GetAllDecksResponse = SquireResponse<Option<HashMap<PlayerId, HashMap<String, Deck>>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPlayerCountRequest {
    pub tourn: TournamentIdentifier,
}

pub type GetPlayerCountResponse = SquireResponse<Option<u64>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLatestPlayerMatchRequest {
    pub tourn: TournamentIdentifier,
    pub player: PlayerIdentifier,
}

pub type GetLatestPlayerMatchResponse = SquireResponse<Option<Option<Option<Round>>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPlayerMatchesRequest {
    pub tourn: TournamentIdentifier,
    pub player: PlayerIdentifier,
}

pub type GetPlayerMatchesResponse = SquireResponse<Option<Option<Vec<Round>>>>;
