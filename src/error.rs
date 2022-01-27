
#[derive(Debug)]
pub enum TournamentError {
    IncorrectStatus,
    PlayerLookup,
    RoundLookup,
    DeckLookup,
    PlayerNotInRound,
    NoActiveRound,
    InvalidBye,
    InvalidGame,
}
