use std::{ops::Deref, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::{
    accounts::SquireAccount,
    admin::Admin,
    identifiers::OpId,
    operations::{
        FullOp, OpLog, OpResult, OpSlice, OpSync, Rollback, SyncError, SyncStatus, TournOp,
    },
    tournament::*,
};

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
    pub fn new(
        owner: SquireAccount,
        seed: TournamentSeed,
    ) -> Self {
        let admin = Admin::new(owner.clone());
        let mut tourn = Tournament::from(seed);
        tourn.admins.insert(admin.id, admin);
        Self {
            tourn,
            log: OpLog::new(owner, seed),
            last_sync: None,
        }
    }

    /// Calculates the number of operations that the manager is storing
    pub fn get_op_count(&self) -> usize {
        self.log.len()
    }

    /// Read only accesses to tournaments don't need to be wrapped, so we can freely provide
    /// references to them
    pub fn tourn(&self) -> &Tournament {
        &self.tourn
    }

    /// Returns the latest active operation id
    pub fn get_last_active_id(&self) -> OpId {
        self.log
            .ops
            .iter()
            .rev()
            .find_map(|op| op.active.then_some(op.id))
            .unwrap()
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
        match self.last_sync {
            Some(id) => self.get_op_slice(id).unwrap().into(),
            None => self.log.ops.clone().into(),
        }
    }

    /// Attempts to sync the given `OpSync` with the operations log. If syncing can occur, the op
    /// logs are merged and SyncStatus::Completed is returned.
    pub fn attempt_sync(&mut self, sy: OpSync) -> SyncStatus {
        let op_id = sy.ops.start_id().ok_or(SyncError::EmptySync)?;
        let slice = self
            .log
            .slice_up_to(op_id)
            .ok_or_else(|| SyncError::UnknownOperation(Box::new(sy.ops.start_op().unwrap())))?;
        let sync = self.log.sync(base, sy);
        if let SyncStatus::Completed(comp) = &sync {
            let mut buffer = self.last_sync_state.clone();
            match comp
                .ops
                .ops
                .iter()
                .filter(|op| op.active)
                .find(|FullOp { op, salt, .. }| buffer.apply_op(*salt, op.clone()).is_err())
            {
                Some(op) => {
                    return SyncStatus::SyncError(Box::new(SyncError::FailedSync(Box::new((
                        op.clone(),
                        comp.clone(),
                    )))));
                }
                None => {
                    // Everything is good!!
                    self.log.overwrite(comp.ops.clone()).unwrap();
                    self.last_sync = Some(self.log.ops.last().unwrap().id);
                    self.tourn = buffer;
                }
            }
        }
        sync
    }

    /// TODO:
    pub fn overwrite(&mut self, ops: OpSlice) -> Result<(), SyncError> {
        let id = ops.ops.first().map(|op| op.id);
        let digest = self.log.overwrite(ops);
        if digest.is_ok() {
            self.last_sync = Some(id.unwrap());
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
    
    fn get_tourn_start(&self) -> Tournament {
        let OpLog { owner, seed, .. } = &self.log;
        let admin = Admin::new(owner.clone());
        let mut tourn = Tournament::from(seed.clone());
        tourn.admins.insert(admin.id, admin);
        tourn
    }

    /// Returns an iterator over all the states of a tournament
    pub fn states(&self) -> StateIter<'_> {
        StateIter {
            state: self.get_tourn_start(),
            ops: self.log.ops.iter(),
            shown_init: false,
        }
    }
}

impl Deref for TournamentManager {
    type Target = Tournament;

    fn deref(&self) -> &Self::Target {
        &self.tourn
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
