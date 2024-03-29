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
    /// The name of the account is already taken by another player in the tournament
    NameTaken,
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
    /// Tried to create a match which contained a player more than once
    RepeatedPlayerInMatch,
    /// Tried to create a match which was either too small or too large. A match should be exactly the
    /// samze size as the tournament's match size.
    IncorrectMatchSize,
    /// The match size was zero (must be nonzero)
    InvalidMatchSize,
    /// The specified min deck count was greater than the max count or visa versa
    InvalidDeckCount,
    /// There is at least one active match without a result
    NoMatchResult,
    /// A player already had the max number of decks
    MaxDecksReached,
    /// Time was added or subtracted such that the time could not be properly stored
    TimeOverflow,
    /// The given name cannot be used as a tournament name
    BadTournamentName,
}

impl fmt::Display for TournamentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TournamentError::*;
        let s = match &self {
            IncorrectStatus(_) => "IncorrectStatus",
            IncorrectRoundStatus(_) => "IncorrectRoundStatus",
            PlayerNotFound => "PlayerNotFound",
            PlayerAlreadyRegistered => "PlayerAlreadyRegistered",
            NameTaken => "NameTaken",
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
            RepeatedPlayerInMatch => "RepeatedPlayerInMatch",
            IncorrectMatchSize => "IncorrectMatchSize",
            InvalidMatchSize => "InvalidMatchSize",
            InvalidDeckCount => "InvalidDeckCount",
            RoundConfirmed => "RoundConfirmed",
            NoMatchResult => "NoMatchResult",
            MaxDecksReached => "MaxDecksReached",
            TimeOverflow => "TimeOverflow",
            BadTournamentName => "BadTournamentName",
        };
        write!(f, "{s}")
    }
}

impl std::error::Error for TournamentError {}
