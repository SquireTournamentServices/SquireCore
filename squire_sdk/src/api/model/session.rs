use serde::{Deserialize, Serialize};
use squire_lib::accounts::SquireAccount;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Credentials {
    Basic { username: String, password: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Login(pub Credentials);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSession;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terminate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reauth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    NotLoggedIn,
    ActiveUser(SquireAccount),
    Guest,
    ExpiredUser(SquireAccount),
    ExpiredGuest,
    UnknownUser,
}
