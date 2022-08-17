use identifiers::{UserAccountID, OrganizationAccountID};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    identifiers::{AdminId, OrganizationAccountId, UserAccountId},
    operations::TournOp,
    settings::TournamentSettingsTree,
    tournament::TournamentPreset,
    tournament_manager::TournamentManager,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SharingPermissions {
    Everything,
    OnlyDeckList,
    OnlyDeckName,
    Nothing
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Platforms {
    Cockatrice,
    MTGOnline,
    Arena
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An enum that encodes the amount of information that is shared about the player after a
/// tournament is over
pub enum SharingPermissions {
    /// Everything about the player is shared and their account is linked to their registration
    /// information
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
    pub gamer_tags: HashMap<Platforms, Option<String>>,
    /// The user's Id
    pub user_id: UserAccountId,
    /// The amount of data that the user wishes to have shared after a tournament is over
    pub do_share: SharingPermissions,
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
    pub default_judge: Vec<SquireAccount>,
    /// A list of accounts that will be added as tournament admins to new tournaments
    pub default_admins: Vec<SquireAccount>,
    /// The default settings for new tournaments
    pub default_tournament_settings: TournamentSettingsTree,
}

impl SquireAccount {
    pub fn new(user_name: String, display_name: String, permissions: SharingPermissions) -> Self {
        SquireAccount {
            display_name,
            user_name,
            gamer_tags: HashMap::new(),
            user_id: Uuid::new_v4().into(),
            permissions
        }
    }

    pub fn add_tag(&mut self, platfrom: Platforms, user_name: String) {
        self.gamer_tags.insert(platfrom, user_name);
    }

    pub fn get_tags(&self, platforms: Vec<Platforms>) -> Vec<Option<String>> {
        let tags = Vec::new()
        for platform in platforms {
            let gamer_tag = self.gamer_tags.get(platform);
            tags.insert(Some(gamer_tag.clone()));
        }
        tags
    }

    pub fn delete_tags(&mut self, platform: Platforms) {
        self.gamer_tags.insert(platform, None)
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
            default_judge: Vec::new(),
            default_admins: Vec::new(),
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
        for judge in self.default_judge.iter().cloned() {
            // Should never error
            let _ = tourn.apply_op(TournOp::RegisterJudge(owner_id, judge));
        }
        for admin in self.default_admins.iter().cloned() {
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
}
