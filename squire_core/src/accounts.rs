use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{get, post, serde::json::Json};

use uuid::Uuid;

use squire_lib::{
    accounts::{OrganizationAccount, SquireAccount},
    identifiers::{AdminId, OrganizationAccountId, UserAccountId},
};

use squire_sdk::{
    accounts::{
        GetAllUsersResponse, GetOrgResponse, GetUserPermissionsResponse, GetUserResponse,
        UpdateOrgAccountRequest, UpdateOrgAccountResponse, UpdateSquireAccountRequest,
        UpdateSquireAccountResponse,
    },
    Action,
};

pub static USERS_MAP: OnceCell<DashMap<UserAccountId, SquireAccount>> = OnceCell::new();
pub static ORGS_MAP: OnceCell<DashMap<OrganizationAccountId, OrganizationAccount>> =
    OnceCell::new();

#[get("/users/get/<id>")]
pub fn users(id: Uuid) -> GetUserResponse {
    GetUserResponse::new(USERS_MAP.get().unwrap().get(&id.into()).map(|a| a.clone()))
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
pub fn user_permissions(id: Uuid) -> GetUserPermissionsResponse {
    GetUserPermissionsResponse::new(
        USERS_MAP
            .get()
            .unwrap()
            .get_mut(&id.into())
            .map(|user| user.permissions),
    )
}

#[post("/users/update/<id>", format = "json", data = "<data>")]
pub fn update_user_account(
    id: Uuid,
    data: Json<UpdateSquireAccountRequest>,
) -> UpdateSquireAccountResponse {
    let mut digest = USERS_MAP.get().unwrap().get_mut(&id.into());

    if let Some(user) = digest.as_mut() {
        if let Some(name) = data.0.display_name {
            user.change_display_name(name);
        }
        for (platform, (action, tag)) in data.0.gamer_tags {
            match action {
                Action::Add => {
                    user.add_tag(platform, tag);
                }
                Action::Delete => {
                    user.delete_tag(&platform);
                }
            }
        }
    }

    UpdateSquireAccountResponse::new(digest.map(|user| user.clone()))
}

#[post("/orgs/update/<id>", format = "json", data = "<data>")]
pub fn update_org_account(
    id: Uuid,
    data: Json<UpdateOrgAccountRequest>,
) -> UpdateOrgAccountResponse {
    let mut digest = ORGS_MAP.get().unwrap().get_mut(&id.into());

    if let Some(org) = digest.as_mut() {
        if let Some(name) = data.0.display_name {
            org.update_display_name(name);
        }
        if let Some(tree) = data.0.default_settings {
            org.default_tournament_settings = tree;
        }
        for (judge, action) in data.0.judges {
            match action {
                Action::Add => {
                    org.update_judges(judge);
                }
                Action::Delete => {
                    org.delete_judge(judge.user_id);
                }
            }
        }
        for (admin, action) in data.0.admins {
            match action {
                Action::Add => {
                    org.update_admins(admin);
                }
                Action::Delete => {
                    org.delete_admin(admin.user_id);
                }
            }
        }
    }

    UpdateOrgAccountResponse::new(digest.map(|org| org.clone()))
}

#[get("/orgs/get/<id>")]
pub fn orgs(id: Uuid) -> GetOrgResponse {
    GetOrgResponse::new(ORGS_MAP.get().unwrap().get(&id.into()).map(|a| a.clone()))
}
