use std::collections::HashSet;

pub use super::{
    error::TournamentError, player::PlayerId, player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
};

#[derive(Debug, Clone)]
pub enum SwissPairingsSettings {
    MatchSize(u8),
    DoCheckIns(bool),
}

#[derive(Debug, Clone)]
pub struct SwissPairings {
    players_per_match: u8,
    do_check_ins: bool,
    check_ins: HashSet<PlayerId>,
}

impl SwissPairings {
    pub fn new(players_per_match: u8) -> Self {
        SwissPairings {
            players_per_match,
            do_check_ins: false,
            check_ins: HashSet::new(),
        }
    }

    pub fn ready_player(&mut self, plyr: PlayerId) {
        self.check_ins.insert(plyr);
    }

    pub fn unready_player(&mut self, plyr: PlayerId) {
        self.check_ins.remove(&plyr);
    }

    pub fn update_settings(&mut self, setting: SwissPairingsSettings) {
        use SwissPairingsSettings::*;
        match setting {
            MatchSize(s) => {
                self.players_per_match = s;
            }
            DoCheckIns(b) => {
                self.do_check_ins = b;
            }
        }
    }
    
    pub fn ready_to_pair(&self, plyr_reg: &PlayerRegistry, rnd_reg: &RoundRegistry) -> bool {
        let mut digest = true;
        if self.do_check_ins {
            digest &= plyr_reg.active_player_count() == self.check_ins.len();
        }
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
