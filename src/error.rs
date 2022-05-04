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
    InvalidGame,
}

impl fmt::Display for TournamentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match &self {
            Self::IncorrectStatus => "IncorrectStatus",
            Self::PlayerLookup => "PlayerLookup",
            Self::RoundLookup => "RoundLookup",
            Self::DeckLookup => "DeckLookup",
            Self::RegClosed => "RegClosed",
            Self::PlayerNotInRound => "PlayerNotInRound",
            Self::NoActiveRound => "NoActiveRound",
            Self::InvalidBye => "InvalidBye",
            Self::InvalidGame => "InvalidGame",
        };
        write!(f, "{}", s)
    }
}

impl std::error::Error for TournamentError {}
