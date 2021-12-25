pub use super::pairing_system::{Arc, MatchRegistry, PairingSystem, PlayerRegistry};

struct SwissPairings {
    players_per_match: u8,
    player_reg: Arc<PlayerRegistry>,
    match_reg: Arc<MatchRegistry>,
}

impl SwissPairings {}

impl PairingSystem for SwissPairings {
    fn new(
        players_per_match: u8,
        player_reg: Arc<PlayerRegistry>,
        match_reg: Arc<MatchRegistry>,
    ) -> Self
    where
        Self: Sized,
    {
        SwissPairings {
            players_per_match,
            player_reg,
            match_reg,
        }
    }
}
