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
