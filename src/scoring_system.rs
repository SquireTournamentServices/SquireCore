pub use crate::match_registry::MatchRegistry;
pub use crate::player_registry::PlayerRegistry;

pub use std::sync::Arc;

pub trait ScoringSystem {
    fn new(player_reg: Arc<PlayerRegistry>, match_reg: Arc<MatchRegistry>) -> Self
    where
        Self: Sized;
}
