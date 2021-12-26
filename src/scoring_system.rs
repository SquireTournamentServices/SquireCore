pub use crate::match_registry::MatchRegistry;
pub use crate::player_registry::PlayerRegistry;

pub use std::sync::{Mutex,Arc};

pub trait ScoringSystem {
    fn new(player_reg: Mutex<Arc<PlayerRegistry>>, match_reg: Mutex<Arc<MatchRegistry>>) -> Self
    where
        Self: Sized;
}
