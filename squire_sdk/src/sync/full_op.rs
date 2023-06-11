use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        identifiers::{id_from_item, PlayerId, RoundId},
        operations::{OpUpdate, TournOp},
    },
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

/// An enum that captures the ways in which two `FullOp`s can differ. This is a vital part in the
/// tournament syncing process.
#[derive(Debug, Clone, Copy)]
pub enum OpDiff {
    /// The two operations are completely equal
    Equal,
    /// The two operations are completely equal
    Time,
    /// The two operations are completely equal
    Different,
}

impl FullOp {
    /// Creates a new FullOp from an existing TournOp
    pub(crate) fn new(op: TournOp) -> Self {
        let salt = Utc::now();
        let id = id_from_item(salt, &op);
        Self {
            op,
            id,
            salt,
        }
    }

    pub(crate) fn get_update(&self) -> OpUpdate {
        self.op.get_update(self.salt)
    }

    pub(crate) fn swap_player_ids(&mut self, old: PlayerId, new: PlayerId) {
        self.op.swap_player_ids(old, new)
    }

    pub(crate) fn swap_round_ids(&mut self, old: RoundId, new: RoundId) {
        self.op.swap_round_ids(old, new)
    }

    /// Calculate the kind of difference (if any) there is between two operations
    pub(crate) fn diff(&self, other: &Self) -> OpDiff {
        if self.op != other.op {
            OpDiff::Different
        } else if self.salt != other.salt {
            OpDiff::Time
        } else {
            OpDiff::Equal
        }
    }
}
