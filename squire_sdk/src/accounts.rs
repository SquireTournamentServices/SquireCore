use crate::response::SquireResponse;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use squire_lib::accounts::{OrganizationAccount, SquireAccount};
pub use squire_lib::identifiers::{UserAccountId, OrganizationAccountId};


pub type GetAllUsersResponse = SquireResponse<HashMap<UserAccountId, SquireAccount>>;

pub type GetUserResponse = SquireResponse<Option<SquireAccount>>;

pub type GetUserPermissionsResponse = SquireResponse<Option<SquireAccount::permissions>>;

pub type GetOrgResponse = SquireResponse<Option<OrganizationAccount>>;

// In SDK, to be used at the body of a POST request
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSquireAccountRequest {
    display_name: Option<String>,
    
    user_name: Option<String>,

    delete_user_name: Option<Boolean>,

    delete_display_name: Option<Boolean>,

    delete_admin: Option<SquireAccount>,
    
    delete_judge: Option<SquireAccount>
  }

pub type UpdateSquireAccountResponse = SquireResponse<UpdateSquireAccountRequest>;