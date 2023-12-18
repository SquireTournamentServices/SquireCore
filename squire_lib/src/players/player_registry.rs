use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Seq};
use TournamentError::{PlayerAlreadyRegistered, PlayerNotFound};

use crate::{
    accounts::SquireAccount,
    error::TournamentError,
    identifiers::PlayerId,
    players::{Player, PlayerStatus},
};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The struct that creates and manages all players.
pub struct PlayerRegistry {
    /// A lookup table between player ids and their names
    // TODO: We don't need this. A GroupMap between PlayerIdentifiers and Players would suffice for
    // the players field
    pub name_and_id: HashMap<String, PlayerId>,
    /// All players in a tournament
    #[serde_as(as = "Seq<(_, _)>")]
    pub players: HashMap<PlayerId, Player>,
    /// A map of players that have checked into the tournament for registration
    pub(crate) check_ins: HashSet<PlayerId>,
}

impl PlayerRegistry {
    /// Creates a new player registry with no players
    pub fn new() -> Self {
        PlayerRegistry {
            name_and_id: HashMap::new(),
            players: HashMap::new(),
            check_ins: HashSet::new(),
        }
    }

    /// Returns a list of copied player ids, this is used in FFI mostly.
    pub fn get_player_ids(&self) -> Vec<PlayerId> {
        self.players.keys().cloned().collect()
    }

    /// Checks in a player for registration
    pub fn check_in(&mut self, id: PlayerId) -> Result<(), TournamentError> {
        if self.players.contains_key(&id) {
            _ = self.check_ins.insert(id);
            Ok(())
        } else {
            Err(PlayerNotFound)
        }
    }

    /// Calculates if a player is registered for the touranment
    pub fn is_registered(&self, id: &PlayerId) -> bool {
        self.players.contains_key(id)
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

    /// Checks if the name is known by the `name_and_id` map in the registry
    fn name_known(&self, name: &str) -> bool {
        self.name_and_id.contains_key(name)
    }

    /// Creates a new player, and attempts to give them the `tourn_name` if the account's user name
    /// is already taken by another player in the tournament. If both of these names are taken, the
    /// same error is returned.
    pub fn register_player_with_name(
        &mut self,
        account: SquireAccount,
        tourn_name: Option<String>,
    ) -> Result<PlayerId, TournamentError> {
        match self.players.get_mut(&(account.id.0.into())) {
            Some(player) => {
                // Re-registering
                player.status = PlayerStatus::Registered;
                Ok(account.id.0.into())
            }
            None => {
                // Not re-registering
                let Some(name) = (!self.name_known(&account.user_name))
                    .then_some(account.get_user_name())
                    .or(tourn_name.filter(|name| !self.name_known(name)))
                else {
                    return Err(TournamentError::NameTaken);
                };
                let plyr = Player::from_account(account);
                let digest = Ok(plyr.id);
                _ = self.name_and_id.insert(name, plyr.id);
                _ = self.players.insert(plyr.id, plyr);
                digest
            }
        }
    }

    /// Creates a new player
    pub fn register_player(&mut self, account: SquireAccount) -> Result<PlayerId, TournamentError> {
        self.register_player_with_name(account, None)
    }

    /// Creates a new player without an account
    pub fn add_guest(
        &mut self,
        salt: DateTime<Utc>,
        name: String,
    ) -> Result<PlayerId, TournamentError> {
        #[allow(clippy::map_entry)]
        if self.name_and_id.contains_key(&name) {
            Err(PlayerAlreadyRegistered)
        } else {
            let mut plyr = Player::new(name.clone());
            plyr.id = Player::create_guest_id(salt, &name);
            let digest = Ok(plyr.id);
            _ = self.name_and_id.insert(name, plyr.id);
            _ = self.players.insert(plyr.id, plyr);
            digest
        }
    }

    /// Creates a new player without an account
    pub fn reregister_guest(&mut self, name: String) -> Result<(), TournamentError> {
        self.name_and_id
            .get(&name)
            .and_then(|id| self.players.get_mut(id))
            .ok_or(PlayerNotFound)?
            .status = PlayerStatus::Registered;
        Ok(())
    }

    /// Sets the specified player's status to `Dropped`
    pub fn drop_player(&mut self, id: &PlayerId) -> Result<(), TournamentError> {
        self.get_mut_player(id)?
            .update_status(PlayerStatus::Dropped);
        Ok(())
    }

    /// Given a player identifier, returns a mutable reference to that player if found
    pub fn get_mut_player(&mut self, id: &PlayerId) -> Result<&mut Player, TournamentError> {
        self.players.get_mut(id).ok_or(PlayerNotFound)
    }

    /// Given a player identifier, returns a reference to that player if found
    pub fn get_player(&self, id: &PlayerId) -> Result<&Player, TournamentError> {
        self.players.get(id).ok_or(PlayerNotFound)
    }

    /// Given a player identifier, returns a reference to that player if found
    pub fn get_by_name(&self, name: &str) -> Result<&Player, TournamentError> {
        self.name_and_id
            .get(name)
            .and_then(|id| self.players.get(id))
            .ok_or(PlayerNotFound)
    }

    /// Given a player identifier, returns that player's id if found
    pub fn get_player_id(&self, name: &str) -> Result<PlayerId, TournamentError> {
        self.name_and_id.get(name).cloned().ok_or(PlayerNotFound)
    }

    /// Given a player identifier, returns that player's name if found
    pub fn get_player_name(&self, id: &PlayerId) -> Option<&String> {
        self.players.get(id).map(|p| &p.name)
    }

    /// Given a player identifier, returns that player's status if found
    pub fn get_player_status(&self, id: &PlayerId) -> Result<PlayerStatus, TournamentError> {
        self.get_player(id).map(|p| p.status)
    }
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        PlayerRegistry::new()
    }
}

#[cfg(test)]
mod tests {

    /// Copied from squire_tests
    fn spoof_account() -> SquireAccount {
        let id = Uuid::new_v4().into();
        SquireAccount {
            id,
            user_name: id.to_string(),
            display_name: id.to_string(),
            gamer_tags: HashMap::new(),
            permissions: SharingPermissions::Everything,
        }
    }

    use std::collections::HashMap;

    use uuid::Uuid;

    use super::PlayerRegistry;
    use crate::{
        accounts::{SharingPermissions, SquireAccount},
        error::TournamentError,
    };

    #[test]
    fn conflicting_names() {
        let mut registry = PlayerRegistry::new();
        let account_one = spoof_account();
        let mut account_two = spoof_account();

        // these two accounts will have conflicting names
        let account_two_previous_name =
            std::mem::replace(&mut account_two.user_name, account_one.get_user_name());

        assert!(registry.register_player(account_one).is_ok());
        assert_eq!(
            registry.register_player(account_two.clone()),
            Err(TournamentError::NameTaken)
        );
        assert!(registry
            .register_player_with_name(account_two, Some(account_two_previous_name))
            .is_ok());
    }
}
