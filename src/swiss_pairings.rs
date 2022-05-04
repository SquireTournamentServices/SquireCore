use std::collections::HashMap;

pub use super::{
    error::TournamentError, player::PlayerId, player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
};

#[derive(Debug, Clone)]
pub struct SwissPairings {
    players_per_match: u8,
}

impl SwissPairings {
    pub fn new(players_per_match: u8) -> Self {
        SwissPairings { players_per_match }
    }
    pub fn ready_player(&mut self, plyr: PlayerId) -> bool {
        todo!()
    }

    pub fn update_settings(&mut self, settings: HashMap<String, String>) -> String {
        todo!()
    }

    pub fn suggest_pairings(
        &self,
        size: u8,
        players: &PlayerRegistry,
        matches: &RoundRegistry,
    ) -> Option<Vec<Vec<PlayerId>>> {
        todo!()
    }

    pub fn rollback_pairings(
        &self,
        players: &mut PlayerRegistry,
        matches: &mut RoundRegistry,
    ) -> Result<(), TournamentError> {
        todo!()
    }
}
