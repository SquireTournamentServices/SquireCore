use std::collections::HashMap;

pub use squire_lib::{
    error::TournamentError,
    player::{Deck, Player, PlayerId},
    player_registry::PlayerIdentifier,
    round::Round,
    tournament::TournamentIdentifier,
};

use crate::response::SquireResponse;

/// The response type used by the `players/<id>/get` SC API. The nested options encode that the
/// requested tournament and player might not be found.
pub type GetPlayerResponse = SquireResponse<Option<Option<Player>>>;

/// The response type used by any SC API that returns multiple players. The option encodes
/// that the requested tournament. The inner data is a map of player Id and player objects.
pub type GetMultiplePlayersResponse = SquireResponse<Option<HashMap<PlayerId, Player>>>;

/// The response type used by the `players/<id>/decks/get/<name>` SC API. The nested options
/// encodes that the requested tournament and player might not be found and, if found, the
/// requested deck might not exist.
pub type GetDeckResponse = SquireResponse<Option<Option<Option<Deck>>>>;

/// The response type used by the `players/<id>/decks/all` SC API. The nested options encodes that
/// the requested tournament and player might not be found
pub type GetAllPlayerDecksResponse = SquireResponse<Option<Option<HashMap<String, Deck>>>>;

/// The response type used by the `decks/all` SC API. The option encodes that the requested
/// tournament. The inner data is a map of all active players and a map of their decks. Note that
/// if a player has no decks, this will just be an empty map.
pub type GetAllDecksResponse = SquireResponse<Option<HashMap<PlayerId, HashMap<String, Deck>>>>;

/// The response type used by any SC API that calculates the number of players in a tournament. The
/// option encodes that the requested tournament.
pub type GetPlayerCountResponse = SquireResponse<Option<u64>>;

/// The response type used by the `players/<id>/latest_match` SC API. The nested options encodes
/// that the requested tournament and player might not be found and, if found, the player might not
/// have been in a round yet.
pub type GetLatestPlayerMatchResponse = SquireResponse<Option<Option<Option<Round>>>>;

/// The response type used by the `players/<id>/matches` SC API. The nested options encodes that
/// the requested tournament and player might not be found. The inner data is an unordered list of
/// rounds.
pub type GetPlayerMatchesResponse = SquireResponse<Option<Option<Vec<Round>>>>;
