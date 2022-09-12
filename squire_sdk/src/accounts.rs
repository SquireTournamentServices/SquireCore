use crate::response::SquireResponse;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use squire_lib::accounts::{OrganizationAccount, SquireAccount};


pub type GetAllUsersResponse = SquireResponse<HashMap<UserAccountId, UserAccount>>;

pub type GetUserResponse = SquireResponse<Option<UserAccount>>;

pub type GetUserPermissionsResponse = SquireResponse<Option<UserAccount::permissions>>;

pub type GetOrgResponse = SquireResponse<Option<OrganizationAccount>>;

// In SDK, to be used at the body of a POST request
pub struct UpdateSquireAccountRequest {
    display_name: Option<String>,
    
    user_name: Option<String>,
    
    admin: Option<SquireAccount>,

    judge: Option<SquireAccount>,

    delete_user_name: Option<Boolean>,

    delete_display_name: Option<Boolean>,

    delete_admin: Option<Boolean>,
    
    delete_judge: Option<Boolean>
  }

pub type UpdateSquireAccountResponse = SquireResponse<UpdateSquireAccountRequest>;