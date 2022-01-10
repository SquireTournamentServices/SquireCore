pub use crate::scoring_system::{
    RoundRegistry, PlayerRegistry, ScoringSystem, Standings,
};

pub struct StandardScoring {}

impl StandardScoring {}

impl ScoringSystem for StandardScoring {
    fn new() -> Self
    where
        Self: Sized,
    {
        StandardScoring {}
    }

    fn get_standings(&self, player_reg: PlayerRegistry, match_reg: RoundRegistry) -> Standings {
        Standings {}
    }
}
