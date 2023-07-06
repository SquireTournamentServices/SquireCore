use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Seq};
use uuid::Uuid;

use crate::{
    admin::Admin,
    identifiers::SquireAccountId,
    tournament::{Tournament, TournamentSeed},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
/// The platforms that we officially support (plus a wildcard)
pub enum Platform {
    /// The Cockatrice platform
    Cockatrice,
    /// The Magic: the Gathering Online platform
    MTGOnline,
    /// The Magic: the Gathering Arena platform
    Arena,
    /// Other platforms that we don't yet support officially
    Other(String),
}

#[derive(
    Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
/// An enum that encodes the amount of information that is shared about the player after a
/// tournament is over
pub enum SharingPermissions {
    /// Everything about the player is shared and their account is linked to their registration
    /// information
    #[default]
    Everything,
    /// Deck information is shared, but not the player's name
    OnlyDeckList,
    /// Only the name of the player's deck is shard
    OnlyDeckName,
    /// Nothing about the player is shared
    Nothing,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
/// The core model for an account for a user
pub struct SquireAccount {
    /// The user's name
    pub user_name: String,
    /// The name that's displayed on the user's account
    pub display_name: String,
    /// The name of the user on MTG Arena
    #[serde_as(as = "Seq<(_, _)>")]
    pub gamer_tags: HashMap<Platform, String>,
    /// The user's Id
    pub id: SquireAccountId,
    /// The amount of data that the user wishes to have shared after a tournament is over
    pub permissions: SharingPermissions,
}

impl SquireAccount {
    /// Creates a new SquireAccount with associated data
    pub fn new(user_name: String, display_name: String) -> Self {
        SquireAccount {
            display_name,
            user_name,
            gamer_tags: HashMap::new(),
            id: SquireAccountId::new(Uuid::new_v4()),
            permissions: SharingPermissions::default(),
        }
    }

    /// Adds a new gamer tag to internal hash map
    pub fn add_tag(&mut self, platfrom: Platform, user_name: String) {
        _ = self.gamer_tags.insert(platfrom, user_name);
    }

    /// Gets the gamer tag the user has for a specific platform
    pub fn get_tag(&mut self, platform: Platform) -> Option<String> {
        self.gamer_tags.get(&platform).cloned()
    }

    /// Gets all tags for a user
    pub fn get_all_tags(&self) -> HashMap<Platform, String> {
        self.gamer_tags.clone()
    }

    /// Deletes a gamer tag for a platform
    pub fn delete_tag(&mut self, platform: &Platform) {
        _ = self.gamer_tags.remove(platform);
    }

    /// Gets the username
    pub fn get_user_name(&self) -> String {
        self.user_name.clone()
    }

    /// Update username to something else
    pub fn change_user_name(&mut self, user_name: String) {
        self.user_name = user_name
    }

    /// Deletes the username
    pub fn delete_user_name(&mut self) {
        self.user_name.clear()
    }

    /// Gets the display name
    pub fn get_display_name(&self) -> String {
        self.display_name.clone()
    }

    /// Update the display name to something else
    pub fn change_display_name(&mut self, display_name: String) {
        self.display_name = display_name
    }

    /// Deletes the display name
    pub fn delete_display_name(&mut self) {
        self.display_name.clear()
    }

    /// Gets the Sharing Permissions
    pub fn get_current_permissions(&self) -> SharingPermissions {
        self.permissions
    }

    /// Update Sharing Permissions to something else
    pub fn change_permissions(&mut self, permissions: SharingPermissions) {
        self.permissions = permissions
    }

    /// Creates a new tournament and loads it with the default settings of the org
    pub fn create_tournament(&self, seed: TournamentSeed) -> Tournament {
        let mut tourn = Tournament::from(seed);
        let admin = Admin::new(self.clone());
        _ = tourn.admins.insert(admin.id, admin);
        tourn
    }
}

impl Hash for SquireAccount {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq for SquireAccount {
    fn eq(&self, other: &Self) -> bool {
        self.user_name == other.user_name
            && self.display_name == other.display_name
            && self.gamer_tags == other.gamer_tags
            && self.id == other.id
            && self.permissions == other.permissions
    }
}
