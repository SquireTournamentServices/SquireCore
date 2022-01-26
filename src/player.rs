use mtgjson::model::deck::Deck;
use uuid::Uuid;

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PlayerStatus {
    SignedUp,
    Registered,
    Dropped,
    Removed,
}

pub struct Player {
    pub uuid: Uuid,
    pub name: String,
    pub game_name: Option<String>,
    decks: HashSet<Deck>,
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

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        &self.uuid == &other.uuid
    }
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            uuid: Uuid::new_v4(),
            name,
            game_name: None,
            decks: HashSet::new(),
            status: PlayerStatus::SignedUp,
        }
    }

    pub fn add_deck(&mut self, deck: Deck) -> () {
        self.decks.insert(deck);
    }

    pub fn update_status(&mut self, status: PlayerStatus) -> () {
        self.status = status;
    }

    pub fn set_game_name(&mut self, name: String) -> () {
        self.game_name = Some(name);
    }

    pub fn can_play(&self) -> bool {
        self.status == PlayerStatus::Registered
    }
}
