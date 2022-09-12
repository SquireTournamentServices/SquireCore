use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::get;

use squire_lib::identifiers::{AdminId, UserAccountId, OrganizationAccountId};
use squire_lib::accounts::{OrganizationAccount, UserAccount};

use squire_sdk::accounts::{
    GetAllUsersResponse, GetOrgResponse, GetUserResponse, GetUserPermissionsResponse, 
    UpdateSquireAccountResponse,
};

pub static USERS_MAP: OnceCell<DashMap<UserAccountId, UserAccount>> = OnceCell::new();
pub static ORGS_MAP: OnceCell<DashMap<OrganizationAccountId, OrganizationAccount>> = OnceCell::new();

#[get("/users/get/<id>")]
pub fn users(id: UserAccountId) -> GetUserResponse {
    GetUserResponse::new(
        USERS_MAP
            .get()
            .unwrap()
            .get(&UserAccountId(id))
            .map(|a| a.clone()),
    )
}

#[get("/users/get/all")]
pub fn all_users() -> GetAllUsersResponse {
    let map = USERS_MAP
        .get()
        .unwrap()
        .iter()
        .map(|r| (r.key().clone(), r.value().clone()))
        .collect();
    GetAllUsersResponse::new(map)
}

#[get("/users/get/<id>/permissions")]
pub fn user_permissions(id: UserAccountId) -> GetUserPermissionsResponse {
    GetUserPermissionsResponse::new(
        ORGS_MAP
        .get()
        .unwrap()
        .get(&UserAccountId(id))
        .map(|user| user.get_current_permissions()),
    )
}

#[post("/users/update/<id>")]
pub fn update_user_account(id: UserAccountId, 
    user_name: Option<String>, 
    display_name: Option<String>,
    delete_user_name: Option<Boolean>,
    delete_display_name: Option<Boolean>
) -> UpdateSquireAccountResponse {
    UpdateSquireAccountResponse::new(
        if(user_name.unwrap() != None) {
            ORGS_MAP
            .get()
            .unwrap()
            .get(&UserAccountId(id))
            .map(|user| user.change_user_name(user_name)),
        }

        if(display_name.unwrap() != None) {
            ORGS_MAP
            .get()
            .unwrap()
            .get(&UserAccountId(id))
            .map(|user| user.change_display_name(display_name)),
        }

        if(delete_user_name.unwrap()) {
            ORGS_MAP
            .get()
            .unwrap()
            .get(&UserAccountId(id))
            .map(|user| user.delete_user_name()),
        }

        if(delete_display_name.unwrap()) {
            ORGS_MAP
            .get()
            .unwrap()
            .get(&UserAccountId(id))
            .map(|user| user.delete_display_name()),
        }
    )
}

#[get("/orgs/get/<id>")]
pub fn orgs(id: OrganizationAccountId) -> GetOrgResponse {
    GetOrgResponse::new(
        ORGS_MAP
            .get()
            .unwrap()
            .get(&OrganizationAccountId(id))
            .map(|a| a.clone()),
    )
}