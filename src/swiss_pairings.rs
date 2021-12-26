pub use super::pairing_system::{Arc, Mutex, MatchRegistry, PairingSystem, PlayerRegistry};

struct SwissPairings {
    players_per_match: u8,
    player_reg: Arc<Mutex<PlayerRegistry>>,
    match_reg: Arc<Mutex<MatchRegistry>>,
}

impl SwissPairings {}

impl PairingSystem for SwissPairings {
    fn new(
        players_per_match: u8,
        player_reg: Arc<Mutex<PlayerRegistry>>,
        match_reg: Arc<Mutex<MatchRegistry>>,
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
