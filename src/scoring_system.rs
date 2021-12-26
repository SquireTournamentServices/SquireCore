pub use crate::standings::Standings;
pub use crate::match_registry::MatchRegistry;
pub use crate::player_registry::PlayerRegistry;

pub use std::sync::{Mutex,Arc};

pub trait ScoringSystem {
    fn new(player_reg: Arc<Mutex<PlayerRegistry>>, match_reg: Arc<Mutex<MatchRegistry>>) -> Self
    where
        Self: Sized;
    
    fn get_standings(&self) -> Standings;
}
