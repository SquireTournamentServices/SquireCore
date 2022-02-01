use crate::error::TournamentError;

use mtgjson::model::deck::Deck;
use uuid::Uuid;

use std::string::ToString;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PlayerStatus {
    SignedUp,
    Registered,
    Dropped,
    Removed,
}

#[derive(Clone)]
pub struct Player {
    pub uuid: Uuid,
    pub name: String,
    pub game_name: Option<String>,
    decks: HashMap<String, Deck>,
    status: PlayerStatus,
}

impl Hash for Player {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let _ = &self.uuid.hash(state);
    }
}

impl ToString for Player {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            uuid: Uuid::new_v4(),
            name,
            game_name: None,
            decks: HashMap::new(),
            status: PlayerStatus::SignedUp,
        }
    }

    pub fn add_deck(&mut self, name: String, deck: Deck) {
        self.decks.insert(name, deck);
    }

    pub fn get_decks(&self) -> HashMap<String, Deck> {
        self.decks.clone()
    }

    pub fn get_deck(&self, name: String) -> Option<Deck> {
        let deck = self.decks.get(&name)?;
        Some((*deck).clone())
    }
    
    pub fn remove_deck(&mut self, name: String) -> Result<(), TournamentError> {
        if self.decks.contains_key(&name) {
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
