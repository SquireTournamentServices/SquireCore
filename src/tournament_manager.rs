use crate::{error::TournamentError, tournament::*};

/// A state manager for the tournament struct
///
/// The manager holds the current tournament and can recreate any meaningful prior state.
///
/// This is the primary synchronization primative between tournaments.
pub struct TournamentManager {
    tourn: Tournament,
    seed: TournamentPreset,
    ops: OpLog,
}

/// This enum captures all ways (methods) in which a tournament can mutate.
pub enum TournOp {}

/// An order list of all operations applied to a tournament
pub struct OpLog {
    ops: Vec<TournOp>,
}

impl TournamentManager {
    /// Read only accesses to tournaments don't need to be wrapped, so we can freely provide
    /// references to them
    pub fn get_state(&self) -> &Tournament {
        &self.tourn
    }

    /// Takes an op log and merges as much of it as possible with this op log.
    /// `Err` is returned if the logs can't be fully merged.
    pub fn merge(&mut self, log: Vec<TournOp>) -> Result<(), TournOp> {
        todo!()
    }

    /// Takes an operation stores it, applies it to the tournament, and returns the result.
    /// NOTE: Even operations that result in a tournament error are stored.
    pub fn apply(&mut self, op: TournOp) -> Result<(), TournamentError> {
        todo!()
    }

    /// Returns an iterator over all the states of a tournament
    pub fn states(&self) -> StateIter {
        todo!()
    }
}

impl TournOp {
    /// Determines if the given operation affects this operation
    pub fn blocks(&self, other: &Self) -> bool {
        todo!()
    }
}

impl PartialEq for TournOp {
    // Needed for determining if two operations are identical
    fn eq(&self, rhs: &Self) -> bool {
        todo!()
    }
}

/// An iterator over all the states of a tournament
pub struct StateIter {}

impl Iterator for StateIter {
    type Item = Tournament;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
