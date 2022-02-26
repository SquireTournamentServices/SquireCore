use crate::error::TournamentError;
use crate::player::{Player, PlayerId, PlayerStatus};

use mtgjson::model::deck::Deck;

use std::{
    collections::{hash_map::Iter, HashMap},
    slice::SliceIndex,
};

#[derive(Debug, Clone)]
pub enum PlayerIdentifier {
    Id(PlayerId),
    Name(String),
}

pub struct PlayerRegistry {
    players: HashMap<PlayerId, Player>,
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        PlayerRegistry::new()
    }
}

impl PlayerRegistry {
    pub fn new() -> Self {
        PlayerRegistry {
            players: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }

    pub fn iter(&self) -> Iter<PlayerId, Player> {
        self.players.iter()
    }

    pub fn add_player(&mut self, name: String) -> Result<PlayerId, TournamentError> {
        if self.verify_identifier(&PlayerIdentifier::Name(name.clone())) {
            Err(TournamentError::PlayerLookup)
        } else {
            let plyr = Player::new(name);
            let id = plyr.id.clone();
            self.players.insert(plyr.id, plyr);
            Ok(id)
        }
    }

    pub fn drop_player(&mut self, ident: PlayerIdentifier) -> Result<(), TournamentError> {
        let plyr = self.get_mut_player(ident)?;
        plyr.update_status(PlayerStatus::Dropped);
        Ok(())
    }

    pub fn remove_player(&mut self, ident: PlayerIdentifier) -> Result<(), TournamentError> {
        let plyr = self.get_mut_player(ident)?;
        plyr.update_status(PlayerStatus::Removed);
        Ok(())
    }

    pub fn get_mut_player(
        &mut self,
        ident: PlayerIdentifier,
    ) -> Result<&mut Player, TournamentError> {
        let id = self.get_player_id(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.players.get_mut(&id).unwrap())
    }

    pub fn get_player(&self, ident: PlayerIdentifier) -> Result<&Player, TournamentError> {
        let id = self.get_player_id(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.players.get(&id).unwrap())
    }

    pub fn get_player_id(&self, ident: PlayerIdentifier) -> Result<PlayerId, TournamentError> {
        match ident {
            PlayerIdentifier::Id(id) => {
                if self.verify_identifier(&PlayerIdentifier::Id(id)) {
                    Ok(id)
                } else {
                    Err(TournamentError::PlayerLookup)
                }
            }
            PlayerIdentifier::Name(name) => {
                let ids: Vec<PlayerId> = self
                    .players
                    .iter()
                    .filter(|(_, p)| p.name == name)
                    .map(|(i, _)| *i)
                    .collect();
                if ids.len() != 1 {
                    Err(TournamentError::PlayerLookup)
                } else {
                    Ok(ids[0])
                }
            }
        }
    }

    pub fn verify_identifier(&self, ident: &PlayerIdentifier) -> bool {
        match ident {
            PlayerIdentifier::Id(id) => self.players.contains_key(id),
            PlayerIdentifier::Name(name) => self.players.iter().any(|(_, p)| p.name == *name),
        }
    }
}
