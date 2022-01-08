pub use crate::match_registry::MatchRegistry;
pub use crate::player_registry::PlayerRegistry;
pub use crate::standings::Standings;

pub trait ScoringSystem {
    fn new() -> Self
    where
        Self: Sized;

    fn get_standings(&self, player_reg: PlayerRegistry, match_reg: MatchRegistry) -> Standings;
}
