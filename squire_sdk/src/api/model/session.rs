use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    Basic {
        username: String,
        password: String,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login(pub Credentials);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSession;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terminate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reauth;
