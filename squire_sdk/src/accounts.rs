#[cfg(feature = "rocket")]
use std::io::Cursor;

#[cfg(feature = "rocket")]
use rocket::{
    http::Status,
    response::{Responder, Result as RResult},
    Response,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "rocket")]
pub const SERIALIZER_ERROR: Status = Status { code: 69 };

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

pub struct AccountResponse<T> {
    pub data: T,
}

#[cfg(feature = "rocket")]
impl<'r, T> Responder<'r, 'r> for AccountResponse<T>
where
    T: Serialize,
{
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> RResult<'r> {
        match serde_json::to_string(&self.data) {
            Err(_) => RResult::Err(SERIALIZER_ERROR),
            Ok(data) => {
                let resp = Response::build()
                    .sized_body(data.len(), Cursor::new(data))
                    .finalize();
                RResult::Ok(resp)
            }
        }
    }
}
