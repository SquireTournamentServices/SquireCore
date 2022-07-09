use crate::{
    error::TournamentError,
    player::{Player, PlayerId, PlayerStatus},
};

use cycle_map::CycleMap;
use mtgjson::model::deck::Deck;

use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    slice::SliceIndex,
};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum PlayerIdentifier {
    Id(PlayerId),
    Name(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRegistry {
    pub name_and_id: CycleMap<String, PlayerId>,
    pub players: HashMap<PlayerId, Player>,
    pub(crate) check_ins: HashSet<PlayerId>,
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
            check_ins: HashSet::new(),
        }
    }

    pub fn check_in(&mut self, id: PlayerId) {
        self.check_ins.insert(id);
    }

    pub fn is_checked_in(&self, id: &PlayerId) -> bool {
        self.check_ins.contains(id)
    }

    pub fn count_check_ins(&self) -> usize {
        self.players
            .iter()
            .filter(|(id, p)| self.is_checked_in(id) && p.can_play())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
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
            let digest = Ok(plyr.id.clone());
            self.name_and_id.insert(name, plyr.id.clone());
            self.players.insert(plyr.id.clone(), plyr);
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
        plyr.update_status(PlayerStatus::Dropped);
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
            PlayerIdentifier::Id(id) => Some(id.clone()),
            PlayerIdentifier::Name(name) => self.name_and_id.get_right(name).cloned(),
        }
    }

    pub fn get_player_status(&self, ident: &PlayerIdentifier) -> Option<PlayerStatus> {
        Some(self.get_player(ident)?.status)
    }

    pub fn verify_identifier(&self, ident: &PlayerIdentifier) -> bool {
        match ident {
            PlayerIdentifier::Id(id) => self.name_and_id.contains_right(id),
            PlayerIdentifier::Name(name) => self.name_and_id.contains_left(name),
        }
    }
}
