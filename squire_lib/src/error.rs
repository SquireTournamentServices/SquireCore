use std::fmt;

use serde::{Deserialize, Serialize};

use crate::tournament::TournamentStatus;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncError {
    IdNotFound,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TournamentError {
    IncorrectStatus(TournamentStatus),
    PlayerLookup,
    RoundLookup,
    DeckLookup,
    RegClosed,
    PlayerNotInRound,
    NoActiveRound,
    InvalidBye,
    ActiveMatches,
    PlayerNotCheckedIn,
    IncompatiblePairingSystem,
    IncompatibleScoringSystem,
}

impl fmt::Display for TournamentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        };
        write!(f, "{}", s)
    }
}

impl std::error::Error for TournamentError {}
