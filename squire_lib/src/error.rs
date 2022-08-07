use std::fmt;

use serde::{Deserialize, Serialize};

use crate::tournament::TournamentStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// An error that encodes problems that can occur during client-server syncing
pub enum SyncError {
    /// The starting operation of the outside log isn't in the local log
    IdNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// All the errors that can occur when apply a tournament operation
pub enum TournamentError {
    /// The tournament has the wrong status
    IncorrectStatus(TournamentStatus),
    /// The specified player couldn't be found
    PlayerLookup,
    /// The specified round couldn't be found
    RoundLookup,
    /// The specified deck couldn't be found
    DeckLookup,
    /// Registration for the tournament is closed
    RegClosed,
    /// The specified player wasn't in the specified round
    PlayerNotInRound,
    /// The specified round was inactive
    NoActiveRound,
    /// A round couldn't be recorded as a bye
    InvalidBye,
    /// The specified player is in an ongoing match
    ActiveMatches,
    /// The specified player was not checked in
    PlayerNotCheckedIn,
    /// The specified setting applies to a pairings system different from the active one
    IncompatiblePairingSystem,
    /// The specified setting applies to a scoring system different from the active one
    IncompatibleScoringSystem,
    /// The specified min deck count was greater than the max count or visa versa
    InvalidDeckCount,
}

impl fmt::Display for TournamentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TournamentError::*;
        let s = match &self {
            IncorrectStatus(_) => "IncorrectStatus",
            PlayerLookup => "PlayerLookup",
            RoundLookup => "RoundLookup",
            DeckLookup => "DeckLookup",
            RegClosed => "RegClosed",
            PlayerNotInRound => "PlayerNotInRound",
            NoActiveRound => "NoActiveRound",
            InvalidBye => "InvalidBye",
            ActiveMatches => "ActiveMatches",
            PlayerNotCheckedIn => "PlayerNotCheckedIn",
            IncompatibleScoringSystem => "IncompatibleScoringSystem",
            IncompatiblePairingSystem => "IncompatiblePairingSystem",
            InvalidDeckCount => "InvalidDeckCount",
        };
        write!(f, "{}", s)
    }
}

impl std::error::Error for TournamentError {}
