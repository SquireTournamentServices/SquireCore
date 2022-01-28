pub use crate::player::Player;
pub use crate::player_registry::PlayerRegistry;
pub use crate::round_registry::RoundRegistry;

pub use std::collections::HashMap;

pub trait Score
where Self: PartialEq<Self> + ToString {
}

pub struct Standings {
    scores: HashMap<Player, dyn Score>,
}

pub trait ScoringSystem {
    fn new() -> Self
    where
        Self: Sized;

    fn get_standings(&self, player_reg: &PlayerRegistry, match_reg: &RoundRegistry) -> Standings;
}
