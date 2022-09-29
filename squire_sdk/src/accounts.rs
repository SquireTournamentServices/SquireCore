use std::collections::HashMap;

use crate::response::SquireResponse;

use serde::{Deserialize, Serialize};
use squire_lib::settings::TournamentSettingsTree;

use crate::Action;

pub use squire_lib::{
    accounts::{OrganizationAccount, Platform, SharingPermissions, SquireAccount},
    identifiers::{OrganizationAccountId, UserAccountId},
};

/// The response type used by the `accounts/users/` SC GET API.
pub type GetAllUsersResponse = SquireResponse<HashMap<UserAccountId, SquireAccount>>;

/// The response type used by the `accounts/users/<id>` SC GET API.
pub type GetUserResponse = SquireResponse<Option<SquireAccount>>;

/// The response type used by the `accounts/users/perms` SC GET API.
pub type GetUserPermissionsResponse = SquireResponse<Option<SharingPermissions>>;

/// The response type used by the `accounts/org/<id>` SC GET API.
pub type GetOrgResponse = SquireResponse<Option<OrganizationAccount>>;

#[derive(Debug, Serialize, Deserialize)]
/// The request type used by the `accounts/user/<id>/update` SC POST API.
pub struct UpdateSquireAccountRequest {
    /// The (potential) new display name of the user
    pub display_name: Option<String>,
    /// Actions to take on gamer tag of the user.
    pub gamer_tags: HashMap<Platform, (Action, String)>,
}

/// The response type used by the `accounts/user/<id>/update` SC POST API.
pub type UpdateSquireAccountResponse = SquireResponse<Option<SquireAccount>>;

#[derive(Debug, Serialize, Deserialize)]
/// The request type used by the `accounts/org/<id>/update` SC POST API.
pub struct UpdateOrgAccountRequest {
    /// The (potential) new display name of the user
    pub display_name: Option<String>,
    /// The (potential) new tournament settings tree
    pub default_settings: Option<TournamentSettingsTree>,
    /// Actions to take on list of default tournament admins of the org.
    pub admins: HashMap<SquireAccount, Action>,
    /// Actions to take on list of default tournament judges of the org.
    pub judges: HashMap<SquireAccount, Action>,
}

/// The response type used by the `accounts/user/<id>/update` SC POST API.
pub type UpdateOrgAccountResponse = SquireResponse<Option<OrganizationAccount>>;
