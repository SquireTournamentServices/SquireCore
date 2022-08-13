use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::get;

use squire_sdk::accounts::{
    AccountId, GetAllUsersResponse, GetOrgResponse, GetUserResponse, OrgAccount, UserAccount,
};
use uuid::Uuid;

pub static USERS_MAP: OnceCell<DashMap<AccountId, UserAccount>> = OnceCell::new();
pub static ORGS_MAP: OnceCell<DashMap<AccountId, OrgAccount>> = OnceCell::new();

#[get("/users/get/<id>")]
pub fn users(id: Uuid) -> GetUserResponse {
    GetUserResponse::new(
        USERS_MAP
            .get()
            .unwrap()
            .get(&AccountId(id))
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

#[get("/orgs/get/<id>")]
pub fn orgs(id: Uuid) -> GetOrgResponse {
    GetOrgResponse::new(
        ORGS_MAP
            .get()
            .unwrap()
            .get(&AccountId(id))
            .map(|a| a.clone()),
    )
}
