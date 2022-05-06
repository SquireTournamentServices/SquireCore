use std::collections::HashSet;

pub use crate::{
    error::TournamentError, player::PlayerId, player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
};

#[derive(Debug, Clone)]
pub enum FluidPairingsSettings {
    MatchSize(u8),
}

#[derive(Debug, Clone)]
pub struct FluidPairings {
    players_per_match: u8,
    check_ins: HashSet<PlayerId>,
}

impl FluidPairings {
    pub fn new(players_per_match: u8) -> Self {
        FluidPairings {
            players_per_match,
            check_ins: HashSet::new(),
        }
    }

    pub fn ready_player(&mut self, plyr: PlayerId) {
        self.check_ins.insert(plyr);
    }

    pub fn unready_player(&mut self, plyr: PlayerId) {
        self.check_ins.remove(&plyr);
    }

    pub fn update_setting(&mut self, setting: FluidPairingsSettings) {
        use FluidPairingsSettings::*;
        match setting {
            MatchSize(s) => {
                self.players_per_match = s;
            }
        }
    }
    
    pub fn ready_to_pair(&self, rnd_reg: &RoundRegistry) -> bool {
        let mut digest = true;
        digest &= rnd_reg.active_round_count() == 0;
        digest
    }

    pub fn pair(
        &mut self,
        players: &PlayerRegistry,
        matches: &RoundRegistry,
    ) -> Option<Vec<Vec<PlayerId>>> {
        todo!()
    }
}
