pub use super::pairing_system::{HashMap, PairingSystem, PlayerRegistry, RoundRegistry, Uuid};

pub struct FluidPairings {
    players_per_match: u8,
}

impl FluidPairings {}

impl PairingSystem for FluidPairings {
    fn new(players_per_match: u8) -> Self
    where
        Self: Sized,
    {
        FluidPairings { players_per_match }
    }
    fn ready_player(&mut self) -> bool {
        todo!()
    }

    fn update_settings(&mut self, settings: HashMap<String, String>) -> String {
        todo!()
    }

    fn suggest_pairings(
        &self,
        players: &PlayerRegistry,
        matches: &RoundRegistry,
    ) -> Result<Vec<Vec<Uuid>>, ()> {
        todo!()
    }

    fn rollback_pairings(
        &self,
        players: &mut PlayerRegistry,
        matches: &mut RoundRegistry,
    ) -> Result<(), ()> {
        todo!()
    }
}
