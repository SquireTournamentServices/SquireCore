use serde::{Deserialize, Serialize};

use super::Credentials;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct RegForm {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

impl From<RegForm> for Credentials {
    fn from(
        RegForm {
            username, password, ..
        }: RegForm,
    ) -> Self {
        Credentials::Basic { username, password }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountCrud;
