use serde::{Deserialize, Serialize};

use crate::{
    accounts::SquireAccount,
    identifiers::{AdminId, JudgeId},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
/// An enum that encodes the ids of tournaments officials
pub enum TournOfficialId {
    /// A judge's id
    Judge(JudgeId),
    /// An admin's id
    Admin(AdminId),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
/// The core model for an account for a tournament judge
pub struct Judge {
    /// The user's name
    pub name: String,
    /// The user's Id
    pub id: JudgeId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
/// The core model for an account for a tournament admin
pub struct Admin {
    /// The user's name
    pub name: String,
    /// The user's Id
    pub id: AdminId,
}

impl Judge {
    /// Creates a new judge object from a `SquireAccount`
    pub fn new(account: SquireAccount) -> Self {
        Self {
            name: account.get_user_name(),
            id: account.get_user_id().0.into(),
        }
    }
}

impl Admin {
    /// Creates a new admin object from a `SquireAccount`
    pub fn new(account: SquireAccount) -> Self {
        Self {
            name: account.get_user_name(),
            id: account.get_user_id().0.into(),
        }
    }
}

impl From<Admin> for Judge {
    fn from(admin: Admin) -> Self {
        Self {
            name: admin.name,
            id: admin.id.0.into(),
        }
    }
}

impl From<JudgeId> for TournOfficialId {
    fn from(id: JudgeId) -> Self {
        Self::Judge(id)
    }
}

impl From<AdminId> for TournOfficialId {
    fn from(id: AdminId) -> Self {
        Self::Admin(id)
    }
}
