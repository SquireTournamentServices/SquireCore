use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{get, post, serde::json::Json};

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

#[post("/users/update/<id>", format = "json", data = "<data>")]
pub fn update_user_account(id: UserAccountId, data: Json<UpdateSquireAccountRequest>) -> UpdateSquireAccountResponse {
    let mut user = ORGS_MAP.get().unwrap().get(&UserAccountId(id));
    UpdateSquireAccountResponse::new(
        if let Some(data.0.0.user_name) = user_name {
            user.change_user_name(user_name);
        }

        if let Some(data.0.0.display_name) = display_name {
            user.change_display_name(display_name);            
        }

    )
}

#[post("/orgs/update/<id>", format - "json", data = "<data>")]
pub fn update_org_account(id: OrganizationAccountId, data: Json<UpdateSquireAccountResponse>) -> UpdateSquireAccountResponse {
    let mut org = ORGS_MAP.get().unwrap().get(&OrganizationAccountId(id));
    UpdateSquireAccountResponse::new(
        if let Some(data.0.user_name) = user_name {
            org.change_user_name(user_name);
        }

        if let Some(data.0.display_name) = display_name {
            org.change_display_name(display_name);            
        }

        if let Some(data.0.0.delete_admin) = delete_admin {
            org.delete_admin(delete_admin);
        }

        if let Some(data.0.0.delete_judge) = delete_judge {
            org.delete_judge(delete_judge);
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