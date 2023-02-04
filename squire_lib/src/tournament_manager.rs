use std::{ops::Deref, slice::Iter};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TournamentManager {
    tourn: Tournament,
    log: OpLog,
    /// The last OpId of the last operation after a successful sync
    last_sync: OpId,
}

impl TournamentManager {
    /// Creates a tournament manager, tournament, and operations log
    pub fn new(
        owner: SquireAccount,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> Self {
        let admin = Admin::new(owner.clone());
        let first_op = FullOp {
            op: TournOp::Create(owner, name.clone(), preset, format.clone()),
            salt: Utc::now(),
            id: Uuid::new_v4().into(),
            active: true,
        };
        let last_sync = first_op.id;
        let mut tourn = Tournament::from_preset(name, preset, format);
        tourn.admins.insert(admin.id, admin);
        Self {
            tourn,
            log: OpLog::new(first_op),
            last_sync,
        }
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
        self.get_op_slice(self.last_sync).unwrap().into()
    }

    /// Attempts to sync the given `OpSync` with the operations log. If syncing can occur, the op
    /// logs are merged and SyncStatus::Completed is returned.
    pub fn attempt_sync(&mut self, sy: OpSync) -> SyncStatus {
        let sync = self.log.sync(sy);
        if let SyncStatus::Completed(c) = &sync {
            let id = c.ops.ops.last().unwrap().id;
            // Can this error? No...
            let _ = self.overwrite(c.ops.clone());
            self.last_sync = id;
        }
        sync
    }

    /// TODO:
    pub fn overwrite(&mut self, ops: OpSlice) -> Result<(), SyncError> {
        let id = ops.ops.first().map(|op| op.id);
        let digest = self.log.overwrite(ops);
        if digest.is_ok() {
            self.last_sync = id.unwrap();
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

    /// Returns an iterator over all the states of a tournament
    pub fn states(&self) -> StateIter<'_> {
        let mut iter = self.log.ops.iter();
        let tourn = match iter.next() {
            Some(FullOp {
                op: TournOp::Create(_, name, seed, format),
                ..
            }) => Tournament::from_preset(name.clone(), *seed, format.clone()),
            _ => {
                unreachable!("First operation isn't a create");
            }
        };
        StateIter {
            state: tourn,
            ops: iter,
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
