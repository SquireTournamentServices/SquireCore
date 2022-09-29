use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use cycle_map::CycleMap;

pub use crate::identifiers::PlayerIdentifier;
use crate::{
    accounts::SquireAccount,
    error::TournamentError,
    identifiers::{PlayerId, PlayerIdentifier::*},
    player::{Player, PlayerStatus},
};

use TournamentError::PlayerLookup;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The struct that creates and manages all players.
pub struct PlayerRegistry {
    /// A lookup table between player ids and their names
    // TODO: We don't need this. A GroupMap between PlayerIdentifiers and Players would suffice for
    // the players field
    pub name_and_id: CycleMap<String, PlayerId>,
    /// All players in a tournament
    pub players: HashMap<PlayerId, Player>,
    /// A map of players that have checked into the tournament for registration
    pub(crate) check_ins: HashSet<PlayerId>,
}

impl PlayerRegistry {
    /// Creates a new player registry with no players
    pub fn new() -> Self {
        PlayerRegistry {
            name_and_id: CycleMap::new(),
            players: HashMap::new(),
            check_ins: HashSet::new(),
        }
    }

    /// Returns a list of copied player ids, this is used in FFI mostly.
    pub fn get_player_ids(&self) -> Vec<PlayerId> {
        self.players.iter().map(|(id, _)| *id).collect()
    }

    /// Checks in a player for registration
    pub fn check_in(&mut self, id: PlayerId) {
        self.check_ins.insert(id);
    }

    /// Confirms that a player is checked in for registration
    pub fn is_checked_in(&self, id: &PlayerId) -> bool {
        self.check_ins.contains(id)
    }

    /// Calculates the number of players that have checked in
    pub fn count_check_ins(&self) -> usize {
        self.players
            .iter()
            .filter(|(id, p)| self.is_checked_in(id) && p.can_play())
            .count()
    }

    /// Calculates if there are no players registered
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Calculates the number of registered players
    pub fn len(&self) -> usize {
        self.players.len()
    }

    /// Calculates the number of registered players that are still active in the tournament
    pub fn active_player_count(&self) -> usize {
        self.players.iter().filter(|(_, p)| p.can_play()).count()
    }

    /* TODO: Is this needed? Was added for sync protocol...
    pub fn import_player(&mut self, plyr: Player) -> Result<(), TournamentError> {
        if self.name_and_id.contains_left(&plyr.name) || self.name_and_id.contains_right(&plyr.id) {
            Err(TournamentError::PlayerLookup)
        } else {
            self.name_and_id.insert(plyr.name.clone(), plyr.id.clone());
            self.players.insert(plyr.id.clone(), plyr);
            Ok(())
        }
    }
    */

    /// Creates a new player
    pub fn add_player(&mut self, account: SquireAccount) -> Result<PlayerId, TournamentError> {
        if self.verify_identifier(&Name(account.get_user_name().clone())) {
            Err(PlayerLookup)
        } else {
            let name = account.get_user_name().clone();
            let plyr = Player::from_account(account);
            let digest = Ok(plyr.id);
            self.name_and_id.insert(name, plyr.id);
            self.players.insert(plyr.id, plyr);
            digest
        }
    }

    /// Creates a new player without an account
    pub fn add_guest(&mut self, name: String) -> Result<PlayerId, TournamentError> {
        if self.verify_identifier(&Name(name.clone())) {
            Err(PlayerLookup)
        } else {
            let plyr = Player::new(name.clone());
            let digest = Ok(plyr.id);
            self.name_and_id.insert(name, plyr.id);
            self.players.insert(plyr.id, plyr);
            digest
        }
    }

    /// Sets the specified player's status to `Dropped`
    pub fn drop_player(&mut self, ident: &PlayerIdentifier) -> Result<(), TournamentError> {
        let plyr = self.get_mut_player(ident)?;
        plyr.update_status(PlayerStatus::Dropped);
        Ok(())
    }

    /// Given a player identifier, returns a mutable reference to that player if found
    pub fn get_mut_player(
        &mut self,
        ident: &PlayerIdentifier,
    ) -> Result<&mut Player, TournamentError> {
        let id = self.get_player_id(ident)?;
        self.players.get_mut(&id).ok_or_else(|| PlayerLookup)
    }

    /// Given a player identifier, returns a reference to that player if found
    pub fn get_player(&self, ident: &PlayerIdentifier) -> Result<&Player, TournamentError> {
        let id = self.get_player_id(ident)?;
        self.players.get(&id).ok_or_else(|| PlayerLookup)
    }

    /// Given a player identifier, returns that player's id if found
    pub fn get_player_id(&self, ident: &PlayerIdentifier) -> Result<PlayerId, TournamentError> {
        match ident {
            Id(id) => Ok(*id),
            Name(name) => self
                .name_and_id
                .get_right(name)
                .cloned()
                .ok_or_else(|| PlayerLookup),
        }
    }

    /// Given a player identifier, returns that player's name if found
    pub fn get_player_name(&self, ident: &PlayerIdentifier) -> Option<String> {
        match ident {
            Name(name) => Some(name.clone()),
            Id(id) => self.name_and_id.get_left(&id).cloned(),
        }
    }

    /// Given a player identifier, returns that player's status if found
    pub fn get_player_status(
        &self,
        ident: &PlayerIdentifier,
    ) -> Result<PlayerStatus, TournamentError> {
        self.get_player(ident).map(|p| p.status)
    }

    /// Verfies that a player's identifier is valid
    pub fn verify_identifier(&self, ident: &PlayerIdentifier) -> bool {
        match ident {
            Id(id) => self.name_and_id.contains_right(id),
            Name(name) => self.name_and_id.contains_left(name),
        }
    }
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        PlayerRegistry::new()
    }
}
