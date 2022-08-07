use serde::{Serialize, Deserialize};

use crate::identifiers::{UserAccountId, OrganizationAccountId};
use crate::settings::TournamentSettingsTree;

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
    Nothing
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// The core model for an account for a user
pub struct SquireAccount {
    pub(crate) display_name: String,
    pub(crate) user_name: String,
    pub(crate) arena_name: Option<String>,
    pub(crate) mtgo_name: Option<String>,
    pub(crate) trice_name: Option<String>,
    pub(crate) user_id: UserAccountId,
    pub(crate) do_share: SharingPermissions,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// The core model for an account for an organization
pub struct OrganizationAccount {
    pub(crate) display_name: String,
    pub(crate) user_name: String,
    pub(crate) user_id: OrganizationAccountId,
    pub(crate) owner: SquireAccount,
    pub(crate) default_judge: Vec<SquireAccount>,
    pub(crate) admin_account: Vec<SquireAccount>,
    pub(crate) default_tournament_settings: TournamentSettingsTree,
}
