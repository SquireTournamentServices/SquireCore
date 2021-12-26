pub use crate::match_registry::MatchRegistry;
pub use crate::player_registry::PlayerRegistry;

pub use std::sync::{Mutex,Arc};

pub trait PairingSystem {
    fn new(
        players_per_match: u8,
        playerReg: Arc<Mutex<PlayerRegistry>>,
        matchReg: Arc<Mutex<MatchRegistry>>,
    ) -> Self
        where Self: Sized;
}
