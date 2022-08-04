use std::{
    collections::HashMap,
    string::ToString,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use mtgjson::model::deck::Deck;

use crate::error::TournamentError;
pub use crate::identifiers::PlayerId;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
#[repr(C)]
pub enum PlayerStatus {
    Registered,
    Dropped,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub game_name: Option<String>,
    pub deck_ordering: Vec<String>,
    pub decks: HashMap<String, Deck>,
    pub status: PlayerStatus,
}

impl Player {
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

    pub fn add_deck(&mut self, name: String, deck: Deck) {
        self.deck_ordering.push(name.clone());
        self.decks.insert(name, deck);
    }

    pub fn get_decks(&self) -> HashMap<String, Deck> {
        self.decks.clone()
    }

    pub fn get_deck(&self, name: &String) -> Option<Deck> {
        let deck = self.decks.get(name)?;
        Some(deck.clone())
    }

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

    pub fn update_status(&mut self, status: PlayerStatus) {
        self.status = status;
    }

    pub fn set_game_name(&mut self, name: String) {
        self.game_name = Some(name);
    }

    pub fn can_play(&self) -> bool {
        self.status == PlayerStatus::Registered
    }
}

impl ToString for Player {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
