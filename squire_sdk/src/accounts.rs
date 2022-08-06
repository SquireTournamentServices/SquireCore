use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::SquireResponse;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct AccountId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub external_id: AccountId,
    pub display_name: String,
    pub account_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgAccount {
    pub external_id: AccountId,
    pub display_name: String,
    pub org_name: String,
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
}

pub type GetUserResponse = SquireResponse<Option<UserAccount>>;

pub type GetAllUsersResponse = SquireResponse<HashMap<AccountId, UserAccount>>;

pub type GetOrgResponse = SquireResponse<Option<OrgAccount>>;
