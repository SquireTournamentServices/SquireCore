use serde::{Deserialize, Serialize};

use crate::model::{accounts::SquireAccount, identifiers::TypeId, tournament::*};

pub mod collections;
pub mod error;
pub mod full_op;
pub mod manager;
pub mod messages;
pub mod processor;
mod utils;

pub use collections::*;
pub use error::*;
pub use full_op::*;
pub use manager::*;
pub use messages::*;

/// The id type for `FullOp`
pub type OpId = TypeId<FullOp>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A struct to help resolve syncing op logs
pub struct OpSync {
    pub(crate) owner: SquireAccount,
    pub(crate) seed: TournamentSeed,
    pub(crate) ops: OpSlice,
}

impl OpSync {
    /// Calculates the length of inner `Vec` of `FullOps`
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if the length of inner `Vec` of `FullOp`s is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Returns the first operation, if it exists. Otherwise, a `SyncError::EmptySync` is
    /// returned.
    pub fn first_op(&self) -> Result<FullOp, SyncError> {
        self.ops.first_op().ok_or(SyncError::EmptySync)
    }

    /// Returns the first operation's id, if it exists. Otherwise, a `SyncError::EmptySync` is
    /// returned.
    pub fn first_id(&self) -> Result<OpId, SyncError> {
        self.ops.first_id().ok_or(SyncError::EmptySync)
    }

    pub fn pop_front(&mut self) -> Result<FullOp, SyncError> {
        self.ops.pop_front().ok_or(SyncError::EmptySync)
    }

    /// Validates the sync against a log. Check id, seed, creator, and len.
    #[allow(dead_code)] // This is a "private" method that is only used by private methods, which
                        // are only used if "client" or "server are enabled. Its easier to just
                        // allow "dead_code" here
    pub(crate) fn validate(&self, log: &OpLog) -> Result<(), SyncError> {
        if self.is_empty() {
            return Err(SyncError::EmptySync);
        }
        let OpLog { owner, seed, .. } = log;
        if self.owner != *owner {
            return Err(Disagreement::new(owner.clone(), self.owner.clone()).into());
        }
        if self.owner != *owner {
            return Err(Disagreement::new(owner.clone(), self.owner.clone()).into());
        }
        if self.seed != *seed {
            return Err(Disagreement::new(seed.clone(), self.seed.clone()).into());
        }
        Ok(())
    }
}
