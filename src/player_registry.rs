use crate::{
    error::TournamentError,
    player::{Player, PlayerId, PlayerStatus},
};

use cycle_map::CycleMap;
use mtgjson::model::deck::Deck;

use serde::{Deserialize, Serialize};

use std::{collections::HashMap, slice::SliceIndex};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum PlayerIdentifier {
    Id(PlayerId),
    Name(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRegistry {
    pub(crate) name_and_id: CycleMap<String, PlayerId>,
    pub(crate) players: HashMap<PlayerId, Player>,
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        PlayerRegistry::new()
    }
}

impl PlayerRegistry {
    pub fn new() -> Self {
        PlayerRegistry {
            name_and_id: CycleMap::new(),
            players: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }

    pub fn active_player_count(&self) -> usize {
        self.players.iter().filter(|(_, p)| p.can_play()).count()
    }

    pub fn add_player(&mut self, name: String) -> Result<PlayerId, TournamentError> {
        if self.verify_identifier(&PlayerIdentifier::Name(name.clone())) {
            Err(TournamentError::PlayerLookup)
        } else {
            let plyr = Player::new(name.clone());
            let digest = Ok(plyr.id);
            self.name_and_id.insert(name, plyr.id);
            self.players.insert(plyr.id, plyr.clone());
            digest
        }
    }

    pub fn drop_player(&mut self, ident: &PlayerIdentifier) -> Option<()> {
        let plyr = self.get_mut_player(ident)?;
        plyr.update_status(PlayerStatus::Dropped);
        Some(())
    }

    pub fn remove_player(&mut self, ident: &PlayerIdentifier) -> Option<()> {
        let plyr = self.get_mut_player(ident)?;
        plyr.update_status(PlayerStatus::Removed);
        Some(())
    }

    pub fn get_mut_player(&mut self, ident: &PlayerIdentifier) -> Option<&mut Player> {
        match ident {
            PlayerIdentifier::Id(id) => self.players.get_mut(id),
            PlayerIdentifier::Name(name) => {
                let id = self.name_and_id.get_right(name)?;
                self.players.get_mut(id)
            }
        }
    }

    pub fn get_player(&self, ident: &PlayerIdentifier) -> Option<&Player> {
        match ident {
            PlayerIdentifier::Id(id) => self.players.get(id),
            PlayerIdentifier::Name(name) => {
                let id = self.name_and_id.get_right(name)?;
                self.players.get(id)
            }
        }
    }

    pub fn get_player_id(&self, ident: &PlayerIdentifier) -> Option<PlayerId> {
        match ident {
            PlayerIdentifier::Id(id) => Some(*id),
            PlayerIdentifier::Name(name) => self.name_and_id.get_right(name).cloned(),
        }
    }

    pub fn verify_identifier(&self, ident: &PlayerIdentifier) -> bool {
        match ident {
            PlayerIdentifier::Id(id) => self.name_and_id.contains_right(id),
            PlayerIdentifier::Name(name) => self.name_and_id.contains_left(name),
        }
    }
}
