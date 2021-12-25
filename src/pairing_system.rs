pub use crate::match_registry::MatchRegistry;
pub use crate::player_registry::PlayerRegistry;

pub use std::sync::Arc;

pub trait PairingSystem {
    fn new(
        players_per_match: u8,
        playerReg: Arc<PlayerRegistry>,
        matchReg: Arc<MatchRegistry>,
    ) -> Self
        where Self: Sized;
}
