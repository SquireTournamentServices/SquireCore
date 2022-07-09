use crate::{
    error::TournamentError,
    operations::{OpLog, OpResult, TournOp},
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

    /// Takes an op log and merges as much of it as possible with this op log.
    /// `Err` is returned if the logs can't be fully merged.
    pub fn merge_logs(&mut self, log: OpLog) -> Result<OpLog, OpLog> {
        todo!()
    }

    /// Takes an operation stores it, applies it to the tournament, and returns the result.
    /// NOTE: Even operations that result in a tournament error are stored.
    pub fn apply(&mut self, op: TournOp) -> OpResult {
        self.log.ops.push(op.clone());
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
    ops: Iter<'a, TournOp>,
    shown_init: bool,
}

impl Iterator for StateIter<'_> {
    type Item = Tournament;

    fn next(&mut self) -> Option<Self::Item> {
        if self.shown_init {
            let op = self.ops.next()?;
            let _ = self.state.apply_op(op.clone());
        } else {
            self.shown_init = true;
        }
        Some(self.state.clone())
    }
}
