use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reauth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terminate;
