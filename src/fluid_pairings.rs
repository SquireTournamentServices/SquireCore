
pub use super::pairing_system::{Arc, MatchRegistry, PairingSystem, PlayerRegistry};

struct FluidPairings {
    players_per_match: u8,
    player_reg: Arc<PlayerRegistry>,
    match_reg: Arc<MatchRegistry>,
}

impl FluidPairings {}

impl PairingSystem for FluidPairings {
    fn new(
        players_per_match: u8,
        player_reg: Arc<PlayerRegistry>,
        match_reg: Arc<MatchRegistry>,
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
