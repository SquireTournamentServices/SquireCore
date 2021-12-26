pub use super::pairing_system::{Arc, MatchRegistry, Mutex, PairingSystem, PlayerRegistry};

struct FluidPairings {
    players_per_match: u8,
    player_reg: Mutex<Arc<PlayerRegistry>>,
    match_reg: Mutex<Arc<MatchRegistry>>,
}

impl FluidPairings {}

impl PairingSystem for FluidPairings {
    fn new(
        players_per_match: u8,
        player_reg: Mutex<Arc<PlayerRegistry>>,
        match_reg: Mutex<Arc<MatchRegistry>>,
    ) -> Self
    where
        Self: Sized,
    {
        FluidPairings {
            players_per_match,
            player_reg,
            match_reg,
        }
    }
}
