pub use crate::scoring_system::{
    Arc, MatchRegistry, Mutex, PlayerRegistry, ScoringSystem, Standings,
};

pub struct StandardScoring {}

impl StandardScoring {}

impl ScoringSystem for StandardScoring {
    fn new(player_reg: Arc<Mutex<PlayerRegistry>>, match_reg: Arc<Mutex<MatchRegistry>>) -> Self
    where
        Self: Sized,
    {
        StandardScoring {}
    }

    fn get_standings(&self) -> Standings {
        Standings {}
    }
}
