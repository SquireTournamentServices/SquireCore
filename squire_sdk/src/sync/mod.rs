use std::slice::Iter;

use serde::{Deserialize, Serialize};

use crate::model::{
    accounts::SquireAccount,
    identifiers::TypeId,
    operations::{OpResult, TournOp},
    tournament::*,
};

pub mod full_op;
pub mod op_align;
pub mod op_log;
pub mod op_sync;
pub mod processor;
pub mod rollback;
pub mod sync_error;
pub mod sync_status;
pub(crate) mod utils;

pub use full_op::*;
pub use op_log::*;
pub use op_sync::*;
pub use rollback::*;
pub use sync_error::*;
pub use sync_status::*;

use self::processor::SyncProcessor;

/// The id type for `FullOp`
pub type OpId = TypeId<FullOp>;

/// A state manager for the tournament struct
///
/// The manager holds the current tournament and can recreate any meaningful prior state.
///
/// This is the primary synchronization primative between tournaments.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TournamentManager {
    tourn: Tournament,
    log: OpLog,
    /// The last OpId of the last operation after a successful sync
    last_sync: Option<OpId>,
}

impl TournamentManager {
    /// Creates a tournament manager, tournament, and operations log
    pub fn new(owner: SquireAccount, seed: TournamentSeed) -> Self {
        let log = OpLog::new(owner, seed);
        let tourn = log.init_tourn();
        Self {
            tourn,
            log,
            last_sync: None,
        }
    }

    pub fn insert(tourn: Tournament) -> Self {
        let name = tourn.name.clone();
        let format = tourn.name.clone();
        Self {
            tourn,
            log: OpLog::new(
                SquireAccount::new("Temp".to_owned(), "Temp".to_owned()),
                TournamentSeed {
                    name,
                    preset: TournamentPreset::Swiss,
                    format,
                },
            ),
            last_sync: None,
        }
    }

    /// Read only accesses to tournaments don't need to be wrapped, so we can freely provide
    /// references to them
    pub fn tourn(&self) -> &Tournament {
        &self.tourn
    }

    /// Takes the manager, removes all unnecessary data for storage, and return the underlying
    /// tournament, consuming the manager in the process.
    pub fn extract(self) -> Tournament {
        self.tourn
    }

    /// Gets a slice of the op log
    fn get_op_slice(&self, id: OpId) -> Option<OpSlice> {
        self.log.get_slice(id)
    }

    /// Gets a slice of the log starting at the operation of the last log.
    /// Primarily used by clients to track when they last synced with the server
    pub fn sync_request(&self) -> OpSync {
        let ops = match self.last_sync {
            Some(id) => self.get_op_slice(id).unwrap(),
            None => self.log.iter().cloned().collect(),
        };
        OpSync {
            owner: self.log.owner.clone(),
            seed: self.log.seed.clone(),
            ops,
        }
    }

    /// Attempts to sync the given `OpSync` with the operations log. If syncing can occur, the op
    /// logs are merged and SyncStatus::Completed is returned.
    pub fn attempt_sync(&mut self, sync: OpSync) -> SyncStatus {
        let mut sync = SyncProcessor::new(sync, &self.log)?;
        match sync.process(&self.log) {
            Ok(()) => SyncStatus::Completed(OpSync {
                owner: self.log.owner.clone(),
                seed: self.log.seed.clone(),
                ops: sync.iter_known().cloned().collect(),
            }),
            Err(err) => SyncStatus::InProgress(Box::new((sync, err))),
        }
    }

    /// TODO:
    pub fn overwrite(&mut self, ops: OpSlice) -> Result<(), SyncError> {
        // FIXME: This should disactive old operations and then add the new operations
        let id = ops.start_id().ok_or(SyncError::EmptySync)?;
        let digest = self.log.overwrite(ops);
        if digest.is_ok() {
            self.last_sync = Some(id);
        }
        digest
    }

    // TODO: Should proposing a rollback lock the tournament manager?
    // No... that seems like it would get complicated as you would need to implement a TTL
    // system...
    /// Creates a rollback proposal but leaves the operations log unaffected.
    /// `id` should be the id of the last operation that is **removed**
    pub fn propose_rollback(&self, id: OpId) -> Option<Rollback> {
        self.log.create_rollback(id)
    }

    /// Takes an operation, ensures all idents are their Id variants, stores the operation, applies
    /// it to the tournament, and returns the result.
    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        let f_op = FullOp::new(op.clone());
        let salt = f_op.salt;
        let digest = self.tourn.apply_op(salt, op);
        if digest.is_ok() {
            self.log.ops.push(f_op);
        }
        digest
    }

    /// Takes an vector of operations and attempts to update the tournament. All operations must
    /// succeed in order for the bulk update the succeed. The update is sandboxed to ensure this.
    pub fn bulk_apply_ops(&mut self, _ops: Vec<TournOp>) -> OpResult {
        todo!()
    }

    /// Returns an iterator over all the states of a tournament
    pub fn states(&self) -> StateIter<'_> {
        StateIter {
            state: self.log.init_tourn(),
            ops: self.log.ops.iter(),
            shown_init: false,
        }
    }
}

#[allow(missing_debug_implementations)]
/// An iterator over all the states of a tournament
pub struct StateIter<'a> {
    state: Tournament,
    ops: Iter<'a, FullOp>,
    shown_init: bool,
}

impl Iterator for StateIter<'_> {
    type Item = Tournament;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: This doesn't take rollbacks into account
        if self.shown_init {
            let op = self.ops.next()?;
            let _ = self.state.apply_op(op.salt, op.op.clone());
        } else {
            self.shown_init = true;
        }
        Some(self.state.clone())
    }
}
