use serde::{Deserialize, Serialize};

use crate::identifiers::{AdminId, JudgeId};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An enum that encodes the ids of tournaments officials
pub enum TournOfficialId {
    /// A judge's id
    Judge(JudgeId),
    /// An admin's id
    Admin(AdminId),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// The core model for an account for a tournament judge
pub struct Judge {
    /// The user's name
    pub name: String,
    /// The user's Id
    pub id: JudgeId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// The core model for an account for a tournament admin
pub struct Admin {
    /// The user's name
    pub name: String,
    /// The user's Id
    pub id: AdminId,
}
