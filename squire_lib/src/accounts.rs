use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};
use serde_with::{Seq, serde_as};
use uuid::Uuid;

use crate::{
    admin::Admin,
    identifiers::SquireAccountId,
    tournament::tournament::Tournament,
};
use crate::tournament::tournament_seed::TournamentSeed;

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
        self.gamer_tags.insert(platfrom, user_name);
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
        self.gamer_tags.remove(platform);
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
        tourn.admins.insert(admin.id, admin);
        tourn
    }
}

/*
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// The core model for an account for an organization
pub struct OrganizationAccount {
    /// The displayed name of the org
    pub display_name: String,
    /// The name of the org
    pub org_name: String,
    /// The org's id
    pub account_id: OrganizationAccountId,
    /// The owner of the account
    pub owner: SquireAccount,
    /// A list of accounts that will be added as judges to new tournaments
    pub default_judges: HashMap<SquireAccountId, SquireAccount>,
    /// A list of accounts that will be added as tournament admins to new tournaments
    pub default_admins: HashMap<SquireAccountId, SquireAccount>,
    /// The default settings for new tournaments
    pub default_tournament_settings: TournamentSettingsTree,
}

impl OrganizationAccount {
    /// Creates a new account object
    pub fn new(owner: SquireAccount, org_name: String, display_name: String) -> Self {
        Self {
            owner,
            org_name,
            display_name,
            account_id: Uuid::new_v4().into(),
            default_judges: HashMap::new(),
            default_admins: HashMap::new(),
            default_tournament_settings: TournamentSettingsTree::new(),
        }
    }

    /// Creates a new tournament and loads it with the default settings of the org
    pub fn create_tournament(&self, seed: TournamentSeed) -> TournamentManager {
        let default_settings = self.default_tournament_settings.as_settings(seed.preset);
        let mut tourn = TournamentManager::new(self.owner.clone(), seed);
        let owner_id: AdminId = self.owner.id.0.into();
        for judge in self.default_judges.values().cloned() {
            // Should never error
            let _ = tourn.apply_op(TournOp::AdminOp(owner_id, AdminOp::RegisterJudge(judge)));
        }
        for admin in self.default_admins.values().cloned() {
            // Should never error
            let _ = tourn.apply_op(TournOp::AdminOp(owner_id, AdminOp::RegisterAdmin(admin)));
        }
        for s in default_settings {
            // TODO: Should we be returning this error??
            // Or maybe this should never error... The settings tree would have to enforce this.
            let _ = tourn.apply_op(TournOp::AdminOp(owner_id, AdminOp::UpdateTournSetting(s)));
        }
        tourn
    }

    /// Update judges
    pub fn update_judges(&mut self, judge: SquireAccount) {
        self.default_judges.insert(judge.id, judge);
    }

    /// Update admins
    pub fn update_admins(&mut self, admin: SquireAccount) {
        self.default_admins.insert(admin.id, admin);
    }

    /// Remove an Admin
    pub fn delete_admin(&mut self, id: SquireAccountId) {
        self.default_admins.remove(&id);
    }

    /// Remove a Judge
    pub fn delete_judge(&mut self, id: SquireAccountId) {
        self.default_judges.remove(&id);
    }

    /// Update Display Name
    pub fn update_display_name(&mut self, display_name: String) {
        self.display_name = display_name
    }

    /// Get Org Name
    pub fn get_org_name(&self) -> String {
        self.org_name.clone()
    }
}
*/

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
