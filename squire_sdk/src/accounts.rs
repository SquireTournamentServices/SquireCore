use crate::response::SquireResponse;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use squire_lib::accounts;


pub type GetAllUsersResponse = SquireResponse<HashMap<UserAccountId, UserAccount>>;

pub type GetUserResponse = SquireResponse<Option<UserAccount>>;

pub type GetUserPermissionsResponse = SquireResponse<Option<UserAccount::permissions>>;

pub type GetOrgResponse = SquireResponse<Option<OrganizationAccount>>;

// In SDK, to be used at the body of a POST request
pub struct UpdateSquireAccountRequest {
    display_name: Option<String>,
    
    user_name: Option<String>,
  }

pub type UpdateSquireAccountResponse = SquireResponse<UpdateSquireAccountRequest>;