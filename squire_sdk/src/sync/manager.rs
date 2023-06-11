use std::{ops::Deref, slice::Iter};

use serde::{Deserialize, Serialize};
use squire_lib::{
    accounts::SquireAccount,
    operations::{OpData, OpResult, TournOp},
    tournament::{Tournament, TournamentSeed},
};

use super::{
    processor::{SyncCompletion, SyncDecision, SyncProcessor},
    FullOp, OpId, OpLog, OpSync, ServerOpLink, SyncError,
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
    pub fn new(owner: SquireAccount, seed: TournamentSeed) -> Self {
        let log = OpLog::new(owner, seed);
        let tourn = log.init_tourn();
        Self {
            tourn,
            log,
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

    /// Method used by clients to create a request for syncing with the remote backend.
    pub fn sync_request(&self) -> OpSync {
        self.log.create_sync_request(self.last_sync)
    }

    /// Attempts to sync the given `OpSync` with the operations log. If syncing can occur, the op
    /// logs are merged and SyncStatus::Completed is returned.
    pub fn init_sync(&mut self, sync: OpSync) -> Result<SyncProcessor, SyncError> {
        SyncProcessor::new(sync, &self.log)
    }

    /// Handles the decision made by the client regarding the sync conflict.
    pub fn handle_decision(&mut self, dec: SyncDecision) -> ServerOpLink {
        match dec {
            SyncDecision::Plucked(proc) => self.process_sync(proc),
            SyncDecision::Purged(comp) => match self.handle_completion(comp.clone()) {
                Ok(()) => comp.into(),
                Err(err) => err.into(),
            },
        }
    }

    /// Processes the SyncProcessor and updated the log if it completes without error
    pub fn process_sync(&mut self, mut proc: SyncProcessor) -> ServerOpLink {
        match (proc.last_known(), self.log.last_id()) {
            (Some(id), None) => return SyncError::UnknownOperation(id).into(),
            (None, None) => {}
            (Some(p_id), Some(l_id)) if p_id == l_id => {}
            (Some(_), Some(_)) | (None, Some(_)) => return SyncError::TournUpdated.into(),
        }
        if proc.last_known().is_none() {
            proc.move_all();
            let comp = proc.finalize();
            return match self.handle_completion(comp.clone()) {
                Ok(()) => comp.into(),
                Err(err) => err.into(),
            };
        }
        let mut buffer = self.clone();
        while let Some(op) = proc.next_op() {
            let Ok(_) = buffer.apply_op_inner(op) else { return proc.into() };
            proc.move_op();
        }
        *self = buffer;
        proc.finalize().into()
    }

    /// This method handles a completed sync request.
    pub fn handle_completion(&mut self, comp: SyncCompletion) -> Result<(), SyncError> {
        match comp {
            SyncCompletion::ForeignOnly(ops) | SyncCompletion::Mixed(ops) => {
                let ops = self.log.apply_unknown(ops)?;
                self.bulk_apply_ops_inner(ops.ops.into_iter())
                    .expect("Non-deterministic tournament operation");
            }
        }
        Ok(())
    }

    /// Takes an operation, ensures all idents are their Id variants, stores the operation, applies
    /// it to the tournament, and returns the result.
    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        self.apply_op_inner(FullOp::new(op.clone()))
    }

    fn apply_op_inner(&mut self, f_op: FullOp) -> OpResult {
        let FullOp { op, salt, .. } = f_op.clone();
        let digest = self.tourn.apply_op(salt, op);
        if digest.is_ok() {
            self.log.ops.push(f_op);
        }
        digest
    }

    /// Takes an vector of operations and attempts to update the tournament. All operations must
    /// succeed in order for the bulk update the succeed. The update is sandboxed to ensure this.
    pub fn bulk_apply_ops(&mut self, ops: Vec<TournOp>) -> OpResult {
        self.bulk_apply_ops_inner(ops.into_iter().map(FullOp::new))
    }

    fn bulk_apply_ops_inner<I>(&mut self, ops: I) -> OpResult
    where
        I: ExactSizeIterator<Item = FullOp>,
    {
        let mut buffer = self.tourn().clone();
        let mut f_ops = Vec::with_capacity(ops.len());
        for f_op in ops {
            let FullOp { op, salt, .. } = f_op.clone();
            buffer.apply_op(salt, op)?;
            f_ops.push(f_op);
        }
        self.log.ops.extend(f_ops);
        self.tourn = buffer;
        Ok(OpData::Nothing)
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
        if self.shown_init {
            let op = self.ops.next()?;
            let _ = self.state.apply_op(op.salt, op.op.clone());
        } else {
            self.shown_init = true;
        }
        Some(self.state.clone())
    }
}

impl Deref for TournamentManager {
    type Target = Tournament;

    fn deref(&self) -> &Self::Target {
        &self.tourn
    }
}
