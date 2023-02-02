use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{rounds::RoundStatus, tournament::TournamentStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// All the errors that can occur when apply a tournament operation
pub enum TournamentError {
    /// The tournament has the wrong status
    IncorrectStatus(TournamentStatus),
    /// The specified player couldn't be found
    PlayerNotFound,
    /// The specified player couldn't be found
    PlayerAlreadyRegistered,
    /// The specified round couldn't be found
    RoundLookup,
    /// The specified tournament official couldn't be found
    OfficalLookup,
    /// The specified deck couldn't be found
    DeckLookup,
    /// The round is already confirmed
    RoundConfirmed,
    /// Registration for the tournament is closed
    RegClosed,
    /// The specified player wasn't in the specified round
    PlayerNotInRound,
    /// The specified player isn't in an active round
    NoActiveRound,
    /// The specified round was inactive
    IncorrectRoundStatus(RoundStatus),
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
    /// There is at least one active match without a result
    NoMatchResult,
}

impl fmt::Display for TournamentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TournamentError::*;
        let s = match &self {
            IncorrectStatus(_) => "IncorrectStatus",
            IncorrectRoundStatus(_) => "IncorrectRoundStatus",
            PlayerNotFound => "PlayerNotFound",
            PlayerAlreadyRegistered => "PlayerAlreadyRegistered",
            RoundLookup => "RoundLookup",
            OfficalLookup => "OfficalLookup",
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
            RoundConfirmed => "RoundConfirmed",
            NoMatchResult => "NoMatchResult",
        };
        write!(f, "{s}")
    }
}

impl std::error::Error for TournamentError {}
