pub use crate::scoring_system::{Arc, MatchRegistry, Mutex, PlayerRegistry, ScoringSystem};

pub struct StandardScoring {}

impl StandardScoring {}

impl ScoringSystem for StandardScoring {
    fn new(player_reg: Mutex<Arc<PlayerRegistry>>, match_reg: Mutex<Arc<MatchRegistry>>) -> Self
    where
        Self: Sized 
    {
        StandardScoring { }
    }
}
