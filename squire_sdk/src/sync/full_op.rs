use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    model::{identifiers::id_from_item, operations::TournOp},
    sync::OpId,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An full operation used by the tournament manager to help track metadata for client-server
/// syncing
pub struct FullOp {
    pub(crate) op: TournOp,
    pub(crate) salt: DateTime<Utc>,
    pub(crate) id: OpId,
}

impl FullOp {
    /// Creates a new FullOp from an existing TournOp
    pub fn new(op: TournOp) -> Self {
        let salt = Utc::now();
        let id = id_from_item(salt, &op);
        Self { op, id, salt }
    }
}
