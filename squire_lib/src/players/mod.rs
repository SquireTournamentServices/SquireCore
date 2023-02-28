use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Seq};
use uuid::Uuid;

pub use mtgjson::model::deck::Deck;

pub use crate::identifiers::PlayerId;
use crate::{accounts::SquireAccount, error::TournamentError, identifiers::id_from_item};

mod player_registry;
pub use player_registry::PlayerRegistry;

#[derive(
    Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord,
)]
#[repr(C)]
/// The registration status of a player
pub enum PlayerStatus {
    /// The player is registered for the tournament
    #[default]
    Registered,
    /// The player has been dropped from the tournament
    Dropped,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The core player model.
/// This model only contains information about the player and what they have registered. All
/// information about their matches, standing, etc is derived externally.
///
/// A player has two primary identifiers, their id and name. They have an opional `game_name` for
/// use in digital tournament where their name on a specific platform might be needed.
pub struct Player {
    /// The player's id
    pub id: PlayerId,
    /// The player's name
    pub name: String,
    /// The player's gamer tag (used for online tournaments)
    pub game_name: Option<String>,
    /// The relative order that the player added their decks (needed for pruning)
    pub deck_ordering: Vec<String>,
    /// The player's registered decks
    #[serde_as(as = "Seq<(_, _)>")]
    pub decks: HashMap<String, Deck>,
    /// The player's status
    pub status: PlayerStatus,
}

impl Player {
    /// Returns a string of a player name and, game name
    pub fn all_names(&self) -> String {
        match &self.game_name {
            Some(game_name) if self.name.eq(game_name) => self.name.clone(),
            Some(game_name) => format!("{} ({game_name})", self.name),
            None => self.name.clone(),
        }
    }

    /// Creates a new player
    pub fn new(name: String) -> Self {
        Player {
            id: Uuid::new_v4().into(),
            name,
            game_name: None,
            deck_ordering: Vec::new(),
            decks: HashMap::new(),
            status: PlayerStatus::Registered,
        }
    }

    pub(crate) fn create_guest_id(salt: DateTime<Utc>, name: &str) -> PlayerId {
        id_from_item(salt, name)
    }

    /// Creates a new player
    pub fn from_account(account: SquireAccount) -> Self {
        Player {
            id: account.id.0.into(),
            name: account.get_user_name(),
            game_name: Some(account.get_display_name()),
            deck_ordering: Vec::new(),
            decks: HashMap::new(),
            status: PlayerStatus::Registered,
        }
    }

    /// Adds a deck to the player
    pub fn add_deck(&mut self, name: String, deck: Deck) {
        self.decks.insert(name.clone(), deck);
        self.deck_ordering.retain(|n| n != &name);
        self.deck_ordering.push(name);
    }

    /// Gets a specific deck from the player
    pub fn get_deck(&self, name: &String) -> Option<&Deck> {
        self.decks.get(name)
    }

    /// Removes a deck from the player
    pub fn remove_deck(&mut self, name: String) -> Result<(), TournamentError> {
        self.decks
            .remove(&name)
            .ok_or(TournamentError::DeckLookup)?;
        self.deck_ordering.retain(|n| n != &name);
        Ok(())
    }

    /// Sets the status of the player
    pub fn update_status(&mut self, status: PlayerStatus) {
        self.status = status;
    }

    /// Calculates if the player is registered
    pub fn can_play(&self) -> bool {
        self.status == PlayerStatus::Registered
    }
}

impl Display for PlayerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayerStatus::Registered => "Registered",
                PlayerStatus::Dropped => "Dropped",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_test() {
        let player = Player::new("Morgan");
        assert_eq!(player.name, "Morgan");
    }

    #[test]
    fn adding_a_deck() {
        let mut player = Player::new("Morgan");
        player.add_deck("Edric", Deck::new("Edric"));
        assert_eq!(player.deck_ordering, vec!["Edric"]);
        
    }
    #[test]
    fn getting_a_deck() {
        let mut player = Player::new("Morgan");
        player.add_deck("Edric", Deck::new("Edric"));
        assert_eq!(player.get_deck("Edric"), Some(&Deck::new("Edric")));
    }
    #[test]
    fn removing_a_deck() { 
        let mut player = Player::new("Morgan");
        player.add_deck("Edric", Deck::new("Edric"));
        player.remove_deck("Edric");
        assert_eq!(player.get_deck("Edric"), None);
    }

    #[test]
    fn updating_status() {
        let mut player = Player::new("Morgan");
        player.update_status(PlayerStatus::Dropped);
        assert_eq!(player.status, PlayerStatus::Dropped);
    }
    #[test]
    fn can_play() {
        let mut player = Player::new("Morgan");
        assert!(!player.can_play());
    }
    #[test]
    fn cant_play_if_dropped() {
        let mut player = Player::new("Morgan");
        player.update_status(PlayerStatus::Dropped);
        assert_eq!(!player.can_play(),false); //I want the test to succeed if this is false
    }
    #[test]
    fn adding_two_decks() {
        let mut player = Player::new("Morgan");
        player.add_deck("Edric", Deck::new("Edric"));
        player.add_deck("Sisay", Deck::new("Sisay"));
        assert_eq!(player.deck_ordering, vec!["Edric", "Sisay"]);
    }
    #[test]
    fn removing_two_decks() {
        let mut player = Player::new("Morgan");
        player.add_deck("Edric", Deck::new("Edric"));
        player.add_deck("Sisay", Deck::new("Sisay"));
        player.remove_deck("Sisay");
        assert_eq!(player.deck_ordering, vec!["Edric"]);
    }
}
