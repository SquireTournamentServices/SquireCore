use std::collections::HashMap;

use crate::response::SquireResponse;

use serde::{Deserialize, Serialize};
use squire_lib::settings::TournamentSettingsTree;

use crate::Action;

pub use squire_lib::{
    accounts::{OrganizationAccount, Platform, SharingPermissions, SquireAccount},
    identifiers::{OrganizationAccountId, SquireAccountId},
};

/// The response type used by the `accounts/register`
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// The name that's displayed on the user's account
    pub user_name: String,
    /// The name that's displayed on the user's account
    pub display_name: String,
}

/// The response type returned by the `account/register`
pub type CreateAccountResponse = SquireResponse<SquireAccount>;

/// The response type used by the `accounts/register`
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    /// The name that's displayed on the user's account
    pub id: SquireAccountId,
}

/// The response type returned by the `account/register`
pub type LoginResponse = SquireResponse<Option<SquireAccount>>;

/// The response type returned by the `accounts/users/` SC GET API.
pub type GetAllUsersResponse = SquireResponse<HashMap<SquireAccountId, SquireAccount>>;

/// The response type returned by the `accounts/users/<id>` SC GET API.
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

/// The error types can be returned during the verification process
#[derive(Debug, Serialize, Deserialize)]
pub enum VerificationError {
    /// A player with that account has already be verified for the tournament
    AlreadyConfirmed,
    /// That account is unknown and likely not valid
    UnknownAccount,
}

/// The body data required to start the verification process
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// The account that is being verified
    pub account: SquireAccount,
}

/// The body data returned during the verification process
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationData {
    /// The verification string
    pub confirmation: String,
    /// The verification status
    pub status: bool,
}

/// The response type used by the `verify` SC GET and POST API.
pub type VerificationResponse = SquireResponse<Result<VerificationData, VerificationError>>;
