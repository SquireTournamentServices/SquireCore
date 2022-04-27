pub use super::pairing_system::{
    HashMap, PairingSystem, PlayerId, PlayerRegistry, RoundRegistry, TournamentError,
};

pub struct SwissPairings {
    players_per_match: u8,
}

impl SwissPairings {
    pub fn new(players_per_match: u8) -> Self {
        SwissPairings { players_per_match }
    }
    fn ready_player(&mut self, plyr: PlayerId) -> bool {
        todo!()
    }

    fn update_settings(&mut self, settings: HashMap<String, String>) -> String {
        todo!()
    }

    fn suggest_pairings(
        &self,
        size: u8,
        players: &PlayerRegistry,
        matches: &RoundRegistry,
    ) -> Option<Vec<Vec<PlayerId>>> {
        todo!()
    }

    fn rollback_pairings(
        &self,
        players: &mut PlayerRegistry,
        matches: &mut RoundRegistry,
    ) -> Result<(), TournamentError> {
        todo!()
    }
}
