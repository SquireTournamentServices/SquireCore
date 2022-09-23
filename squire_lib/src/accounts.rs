use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    identifiers::{AdminId, OrganizationAccountId, UserAccountId},
    operations::TournOp,
    settings::TournamentSettingsTree,
    tournament::TournamentPreset,
    tournament_manager::TournamentManager,
};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
/// The platforms that we officially support (plus a wildcard)
pub enum Platforms {
    /// The Cockatrice platform
    Cockatrice,
    /// The Magic: the Gathering Online platform
    MTGOnline,
    /// The Magic: the Gathering Arena platform
    Arena,
    /// Other platforms that we don't yet support officially
    Other(String),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// The core model for an account for a user
pub struct SquireAccount {
    /// The user's name
    pub user_name: String,
    /// The name that's displayed on the user's account
    pub display_name: String,
    /// The name of the user on MTG Arena
    pub gamer_tags: HashMap<Platforms, String>,
    /// The user's Id
    pub user_id: UserAccountId,
    /// The amount of data that the user wishes to have shared after a tournament is over
    pub permissions: SharingPermissions,
}

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
    pub default_judges: HashMap<UserAccountId, SquireAccount>,
    /// A list of accounts that will be added as tournament admins to new tournaments
    pub default_admins: HashMap<UserAccountId, SquireAccount>,
    /// The default settings for new tournaments
    default_tournament_settings: TournamentSettingsTree,
}

impl SquireAccount {
    /// Creates a new SquireAccount with associated data
    pub fn new(user_name: String, display_name: String) -> Self {
        SquireAccount {
            display_name,
            user_name,
            gamer_tags: HashMap::new(),
            user_id: Uuid::new_v4().into(),
            permissions: SharingPermissions::default(),
        }
    }

    /// Adds a new gamer tag to internal hash map
    pub fn add_tag(&mut self, platfrom: Platforms, user_name: String) {
        self.gamer_tags.insert(platfrom, user_name);
    }

    /// Gets the gamer tag the user has for a specific platform
    pub fn get_tag(&mut self, platform: Platforms) -> Option<String> {
        self.gamer_tags.get(&platform).cloned()
    }

    /// Gets all tags for a user
    pub fn get_all_tags(&self) -> HashMap<Platforms, String> {
        self.gamer_tags.clone()
    }

    /// Deletes a gamer tag for a platform
    pub fn delete_tag(&mut self, platform: &Platforms) {
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

    /// Gets the user ID
    pub fn get_user_id(&self) -> UserAccountId {
        self.user_id
    }

    /// Gets the Sharing Permissions
    pub fn get_current_permissions(&self) -> SharingPermissions {
        self.permissions.clone()
    }

    /// Update Sharing Permissions to something else
    pub fn change_permissions(&mut self, permissions: SharingPermissions) {
        self.permissions = permissions
    }

    /// Creates a new tournament and loads it with the default settings of the org
    pub fn create_tournament(
        &self,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentManager {
        TournamentManager::new(self.clone(), name, preset, format)
    }
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
    pub fn create_tournament(
        &self,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentManager {
        let mut tourn = TournamentManager::new(self.owner.clone(), name, preset, format);
        let owner_id: AdminId = self.owner.user_id.0.into();
        for judge in self.default_judges.values().cloned() {
            // Should never error
            let _ = tourn.apply_op(TournOp::RegisterJudge(owner_id, judge));
        }
        for admin in self.default_admins.values().cloned() {
            // Should never error
            let _ = tourn.apply_op(TournOp::RegisterJudge(owner_id, admin));
        }
        for s in self.default_tournament_settings.as_settings(preset) {
            // TODO: Should we be returning this error??
            // Or maybe this should never error... The settings tree would have to enforce this.
            let _ = tourn.apply_op(TournOp::UpdateTournSetting(owner_id, s));
        }
        tourn
    }

    /// Update judges
    pub fn update_judges(&mut self, judge: SquireAccount) {
        self.default_judges.insert(judge.user_id, judge);
    }

    /// Update admins
    pub fn update_admins(&mut self, admin: SquireAccount) {
        self.default_admins.insert(admin.user_id, admin);
    }

    /// Remove an Admin
    pub fn delete_admin(&mut self, id: UserAccountId) {
        self.default_admins.remove(&id);
    }

    /// Remove a Judge
    pub fn delete_judge(&mut self, id: UserAccountId) {
        self.default_judges.remove(&id);
    }

    /// Update Display Name
    pub fn update_display_name(&mut self, display_name: String) {
        self.display_name = display_name
    }

    /// Get Org Name
    pub fn get_org_name(&self) -> String {
        self.org_name.clone();
    }
}
