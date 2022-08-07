use std::collections::HashMap;

use mtgjson::model::deck::Deck;
pub use squire_lib::{
    error::TournamentError,
    player::{Player, PlayerId},
    player_registry::PlayerIdentifier,
    round::Round,
    tournament::TournamentIdentifier,
};

use crate::response::SquireResponse;

pub type GetPlayerResponse = SquireResponse<Option<Option<Player>>>;

pub type GetMultiplePlayersResponse = SquireResponse<Option<HashMap<PlayerId, Player>>>;

pub type GetDeckResponse = SquireResponse<Option<Option<Option<Deck>>>>;

pub type GetAllPlayerDecksResponse = SquireResponse<Option<Option<HashMap<String, Deck>>>>;

pub type GetAllDecksResponse = SquireResponse<Option<HashMap<PlayerId, HashMap<String, Deck>>>>;

pub type GetPlayerCountResponse = SquireResponse<Option<u64>>;

pub type GetLatestPlayerMatchResponse = SquireResponse<Option<Option<Option<Round>>>>;

pub type GetPlayerMatchesResponse = SquireResponse<Option<Option<Vec<Round>>>>;
