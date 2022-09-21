use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use mtgjson::model::deck::Deck;

pub use crate::identifiers::PlayerId;
use crate::{accounts::SquireAccount, error::TournamentError};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, Hash)]
#[repr(C)]
/// The registration status of a player
pub enum PlayerStatus {
    /// The player is registered for the tournament
    Registered,
    /// The player has been dropped from the tournament
    Dropped,
}

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
    pub decks: HashMap<String, Deck>,
    /// The player's status
    pub status: PlayerStatus,
}

impl Player {
    /// Creates a new player
    pub fn new(name: String) -> Self {
        Player {
            id: PlayerId::new(Uuid::new_v4()),
            name,
            game_name: None,
            deck_ordering: Vec::new(),
            decks: HashMap::new(),
            status: PlayerStatus::Registered,
        }
    }

    /// Creates a new player
    pub fn from_account(account: SquireAccount) -> Self {
        Player {
            id: account.get_user_id().0.into(),
            name: account.get_user_name(),
            game_name: Some(account.get_display_name()),
            deck_ordering: Vec::new(),
            decks: HashMap::new(),
            status: PlayerStatus::Registered,
        }
    }

    /// Adds a deck to the player
    pub fn add_deck(&mut self, name: String, deck: Deck) {
        self.deck_ordering.push(name.clone());
        self.decks.insert(name, deck);
    }

    /// Gets a specific deck from the player
    pub fn get_deck(&self, name: &String) -> Option<Deck> {
        let deck = self.decks.get(name)?;
        Some(deck.clone())
    }

    /// Removes a deck from the player
    pub fn remove_deck(&mut self, name: String) -> Result<(), TournamentError> {
        if self.decks.contains_key(&name) {
            let index = self.deck_ordering.iter().position(|n| n == &name).unwrap();
            self.deck_ordering.remove(index);
            self.decks.remove(&name);
            Ok(())
        } else {
            Err(TournamentError::DeckLookup)
        }
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
