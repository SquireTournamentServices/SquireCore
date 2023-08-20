use serde::{Deserialize, Serialize};

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
