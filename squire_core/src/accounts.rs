use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{get, serde::json::Json};

use squire_sdk::accounts::{
    AccountId, AccountIdentifier, GetAllUsersResponse, GetOrgResponse, GetUserResponse, OrgAccount,
    UserAccount,
};

pub static USERS_MAP: OnceCell<DashMap<AccountId, UserAccount>> = OnceCell::new();
pub static ORGS_MAP: OnceCell<DashMap<AccountId, OrgAccount>> = OnceCell::new();

#[get("/users", format = "json", data = "<ident>")]
pub fn users(ident: Json<AccountIdentifier>) -> GetUserResponse {
    match ident.0 {
        AccountIdentifier::Id(id) => {
            GetUserResponse::new(USERS_MAP.get().unwrap().get(&id).map(|a| a.clone()))
        }
        AccountIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/users/all")]
pub fn all_users() -> GetAllUsersResponse {
    let map = USERS_MAP
        .get()
        .unwrap()
        .iter()
        .map(|r| (r.key().clone(), r.value().clone()))
        .collect();
    GetAllUsersResponse::new(map)
}

#[get("/orgs", format = "json", data = "<ident>")]
pub fn orgs(ident: Json<AccountIdentifier>) -> GetOrgResponse {
    match ident.0 {
        AccountIdentifier::Id(id) => {
            GetOrgResponse::new(ORGS_MAP.get().unwrap().get(&id).map(|a| a.clone()))
        }
        AccountIdentifier::Name(_name) => {
            todo!("Yet to be impl-ed");
        }
    }
}
