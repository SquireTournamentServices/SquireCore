use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{get, serde::json::Json};

use squire_sdk::accounts::{
    AccountId, AccountIdentifier, AccountResponse, OrgAccount, UserAccount,
};

pub static USERS_MAP: OnceCell<DashMap<AccountId, UserAccount>> = OnceCell::new();
pub static ORGS_MAP: OnceCell<DashMap<AccountId, OrgAccount>> = OnceCell::new();

#[get("/user", format = "json", data = "<ident>")]
pub fn user(ident: Json<AccountIdentifier>) -> AccountResponse<Option<UserAccount>> {
    match ident.0 {
        AccountIdentifier::Id(id) => AccountResponse {
            data: USERS_MAP.get().unwrap().get(&id).map(|a| a.clone()),
        },
        AccountIdentifier::Name(name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/orgs", format = "json", data = "<ident>")]
pub fn orgs(ident: Json<AccountIdentifier>) -> AccountResponse<Option<OrgAccount>> {
    match ident.0 {
        AccountIdentifier::Id(id) => AccountResponse {
            data: ORGS_MAP.get().unwrap().get(&id).map(|a| a.clone()),
        },
        AccountIdentifier::Name(name) => {
            todo!("Yet to be impl-ed");
        }
    }
}
