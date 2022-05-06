use std::fmt;

#[derive(Debug)]
pub enum TournamentError {
    IncorrectStatus,
    PlayerLookup,
    RoundLookup,
    DeckLookup,
    RegClosed,
    PlayerNotInRound,
    NoActiveRound,
    InvalidBye,
    ActiveMatches,
    PlayerNotCheckedIn,
}

impl fmt::Display for TournamentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TournamentError::*;
        let s = match &self {
            IncorrectStatus => "IncorrectStatus",
            PlayerLookup => "PlayerLookup",
            RoundLookup => "RoundLookup",
            DeckLookup => "DeckLookup",
            RegClosed => "RegClosed",
            PlayerNotInRound => "PlayerNotInRound",
            NoActiveRound => "NoActiveRound",
            InvalidBye => "InvalidBye",
            ActiveMatches => "ActiveMatches",
            PlayerNotCheckedIn => "PlayerNotCheckedIn",
        };
        write!(f, "{}", s)
    }
}

impl std::error::Error for TournamentError {}
