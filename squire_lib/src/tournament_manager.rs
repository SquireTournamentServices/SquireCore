#![allow(unused)]
use std::{
    collections::{HashMap, HashSet},
    slice::Iter,
};

use serde::{Deserialize, Serialize};

use cycle_map::CycleMap;
use uuid::Uuid;

use crate::{
    error::TournamentError,
    identifiers::{OpId, PlayerIdentifier, RoundIdentifier},
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
#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentManager {
    tourn: Tournament,
    log: OpLog,
    /// The last OpId of the last operation after a successful sync
    last_sync: OpId,
}

impl TournamentManager {
    /// Creates a tournament manager, tournament, and operations log
    pub fn new(name: String, preset: TournamentPreset, format: String) -> Self {
        let first_op = FullOp {
            op: TournOp::Create(name.clone(), preset, format.clone()),
            id: Uuid::new_v4().into(),
            active: true,
        };
        let last_sync = first_op.id;
        let tourn = Tournament::from_preset(name, preset, format);
        Self {
            tourn,
            log: OpLog::new(first_op),
            last_sync,
        }
    }

    /// Read only accesses to tournaments don't need to be wrapped, so we can freely provide
    /// references to them
    pub fn get_state(&self) -> &Tournament {
        &self.tourn
    }
    
    /// Returns the latest active operation id
    pub fn get_last_active_id(&self) -> OpId {
        self.log.ops.iter().rev().find_map(|op| if op.active { Some(op.id) } else { None } ).unwrap()
    }

    /// Takes the manager, removes all unnecessary data for storage, and return the underlying
    /// tournament, consuming the manager in the process.
    pub fn extract(mut self) -> Tournament {
        self.tourn.player_reg.check_ins = HashSet::new();
        self.tourn.player_reg.name_and_id = CycleMap::new();
        for (_, plyr) in self.tourn.player_reg.players.iter_mut() {
            plyr.deck_ordering = Vec::new();
        }
        self.tourn.round_reg.num_and_id = CycleMap::new();
        self.tourn.round_reg.opponents = HashMap::new();
        for (_, rnd) in self.tourn.round_reg.rounds.iter_mut() {
            rnd.confirmations = HashSet::new();
        }
        self.tourn
    }

    /// Gets a slice of the op log
    pub fn get_op_slice(&self, id: OpId) -> Option<OpSlice> {
        self.log.get_slice(id)
    }

    /// Gets a slice of the log starting at the operation of the last log.
    /// Primarily used by clients to track when they last synced with the server
    pub fn slice_from_last_sync(&self) -> OpSlice {
        self.get_op_slice(self.last_sync).unwrap()
    }

    /// Attempts to sync the given `OpSync` with the operations log. If syncing can occur, the op
    /// logs are merged and SyncStatus::Completed is returned.
    pub fn attempt_sync(&mut self, sy: OpSync) -> SyncStatus {
        // TODO: This should move the last_sync id forward
        self.log.sync(sy)
    }

    /// TODO:
    pub fn overwrite(&mut self, ops: OpSlice) -> Result<(), SyncError> {
        // TODO: This should set the last_sync id back
        self.log.overwrite(ops)
    }

    // TODO: Should proposing a rollback lock the tournament manager?
    /// Creates a rollback proposal but leaves the operations log unaffected.
    /// `id` should be the id of the last operation that is **removed**
    pub fn propose_rollback(&self, id: OpId) -> Option<Rollback> {
        self.log.create_rollback(id)
    }

    /// Takes an operation, ensures all idents are their Id variants, stores the operation, applies
    /// it to the tournament, and returns the result.
    /// NOTE: That an operation is always stored, regardless of the outcome
    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        // NOTE: We need to ensure that common data is being stored in the op log, i.e. player
        // names and match numbers instead of IDs. IDs aren't shared between clients.
        let op = if let Some(ident) = op.get_player_ident() {
            let name = self
                .tourn
                .player_reg
                .get_player_name(&ident)
                .ok_or(TournamentError::PlayerLookup)?;
            op.swap_player_ident(PlayerIdentifier::Name(name))
        } else if let Some(idents) = op.list_player_ident() {
            let mut new_idents = Vec::with_capacity(idents.len());
            for ident in idents {
                new_idents.push(PlayerIdentifier::Name(
                    self.tourn
                        .player_reg
                        .get_player_name(&ident)
                        .ok_or(TournamentError::PlayerLookup)?,
                ));
            }
            op.swap_all_player_idents(new_idents)
        } else if let Some(ident) = op.get_match_ident() {
            let num = self
                .tourn
                .round_reg
                .get_round_number(&ident)
                .ok_or(TournamentError::PlayerLookup)?;
            op.swap_match_ident(RoundIdentifier::Number(num))
        } else {
            op
        };
        let f_op = FullOp::new(op.clone());
        self.log.ops.push(f_op);
        self.tourn.apply_op(op)
    }

    /// Returns an iterator over all the states of a tournament
    pub fn states(&self) -> StateIter<'_> {
        let mut iter = self.log.ops.iter();
        let tourn = match iter.next() {
            Some(FullOp {
                op: TournOp::Create(name, seed, format),
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
            let _ = self.state.apply_op(op.op.clone());
        } else {
            self.shown_init = true;
        }
        Some(self.state.clone())
    }
}
