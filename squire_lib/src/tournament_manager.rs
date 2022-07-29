use crate::{
    error::TournamentError,
    operations::{FullOp, OpLog, OpResult, OpSlice, OpSync, Rollback, SyncStatus, Synced, TournOp},
    player_registry::PlayerIdentifier,
    round_registry::RoundIdentifier,
    tournament::*,
};

use serde::{Deserialize, Serialize};

use std::slice::Iter;

/// A state manager for the tournament struct
///
/// The manager holds the current tournament and can recreate any meaningful prior state.
///
/// This is the primary synchronization primative between tournaments.
#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentManager {
    tourn: Tournament,
    seed: TournamentPreset,
    name: String,
    format: String,
    log: OpLog,
}

impl TournamentManager {
    /// Read only accesses to tournaments don't need to be wrapped, so we can freely provide
    /// references to them
    pub fn get_state(&self) -> &Tournament {
        &self.tourn
    }

    /// Takes the manager and return the underlying tournament, consuming the manager in the
    /// process.
    pub fn extract(self) -> Tournament {
        self.tourn
    }

    /// Starts the syncing processing. If syncing can occur, the op logs are merged and
    /// SyncStatus::Completed is returned.
    pub fn start_sync(&mut self, sy: OpSync) -> SyncStatus {
        todo!()
    }

    /// Imports a synced op log. Returns Ok containing the given Synced if this log hasn't changed.
    /// Returns Err containing a SyncStatus if it the log has changed. If that SyncStatus is
    /// Completed, the this log is updated accordingly (otherwise, the method would have to be
    /// immediately called again).
    pub fn import_sync(&mut self, ops: Synced) -> Result<Synced, SyncStatus> {
        todo!()
    }

    pub fn overwrite(&mut self, ops: OpSlice) {
        todo!()
    }

    pub fn propose_rollback<F>(&mut self, f: F) -> Rollback
    where
        F: FnMut(&FullOp) -> Option<bool>,
    {
        todo!()
    }

    /// Takes an operation, ensures all idents are their Id variants, stores the operation, applies
    /// it to the tournament, and returns the result.
    /// NOTE: That if an operation can be convert, it is always stored, regardless of the outcome
    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        let op = if let Some(ident) = op.get_player_ident() {
            let id = self
                .tourn
                .player_reg
                .get_player_id(&ident)
                .ok_or(TournamentError::PlayerLookup)?;
            op.swap_player_ident(PlayerIdentifier::Id(id))
        } else if let Some(idents) = op.list_player_ident() {
            let mut new_idents = Vec::with_capacity(idents.len());
            for ident in idents {
                new_idents.push(PlayerIdentifier::Id(
                    self.tourn
                        .player_reg
                        .get_player_id(&ident)
                        .ok_or(TournamentError::PlayerLookup)?,
                ));
            }
            op.swap_all_player_idents(new_idents)
        } else if let Some(ident) = op.get_match_ident() {
            let id = self
                .tourn
                .round_reg
                .get_round_id(&ident)
                .ok_or(TournamentError::PlayerLookup)?;
            op.swap_match_ident(RoundIdentifier::Id(id))
        } else {
            op
        };
        let f_op = FullOp::new(op.clone());
        self.log.ops.push(f_op);
        self.tourn.apply_op(op)
    }

    /// Returns an iterator over all the states of a tournament
    pub fn states(&self) -> StateIter {
        StateIter {
            state: Tournament::from_preset(
                self.name.clone(),
                self.seed.clone(),
                self.format.clone(),
            ),
            ops: self.log.ops.iter(),
            shown_init: false,
        }
    }
}

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
            let _ = self.state.apply_op(op.op.clone());
        } else {
            self.shown_init = true;
        }
        Some(self.state.clone())
    }
}
