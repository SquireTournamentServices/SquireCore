use std::io::Cursor;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rocket::{
    get,
    serde::json::Json,
    http::{self, Status},
    response::{Responder, Result as RResult},
    Response,
};


pub const SERIALIZER_ERROR: Status = Status { code: 69 };

pub static USERS_MAP: OnceCell<DashMap<AccountId, UserAccount>> = OnceCell::new();
pub static ORGS_MAP:  OnceCell<DashMap<AccountId, OrgAccount>>  = OnceCell::new();

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct AccountId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountIdentifier {
    Name(String),
    Id(AccountId),
}

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

#[get("/user", format = "json", data = "<ident>")]
pub fn user(ident: Json<AccountIdentifier>) -> AccountResponse<Option<UserAccount>> {
    match ident.0 {
        AccountIdentifier::Id(id) => {
            AccountResponse {
                data: USERS_MAP.get().unwrap().get(&id).map(|a| a.clone())
            }
        }
        AccountIdentifier::Name(name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

#[get("/orgs", format = "json", data = "<ident>")]
pub fn orgs(ident: Json<AccountIdentifier>) -> AccountResponse<Option<OrgAccount>> {
    match ident.0 {
        AccountIdentifier::Id(id) => {
            AccountResponse {
                data: ORGS_MAP.get().unwrap().get(&id).map(|a| a.clone())
            }
        }
        AccountIdentifier::Name(name) => {
            todo!("Yet to be impl-ed");
        }
    }
}

pub struct AccountResponse<T> {
    data: T,
}

impl<'r, T> Responder<'r, 'r> for AccountResponse<T>
where
    T: Serialize,
{
    fn respond_to(self, request: &'r rocket::Request<'_>) -> RResult<'r> {
        match serde_json::to_string(&self.data) {
            Err(_) => RResult::Err(SERIALIZER_ERROR),
            Ok(data) => {
                let mut resp = Response::build().sized_body(data.len(), Cursor::new(data)).finalize();
                RResult::Ok(resp)
            }
        }
    }
}
