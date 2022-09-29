use crate::{
    identifiers::PlayerId,
    pairings::{PairingAlgorithm, Pairings},
    player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
    scoring::{Score, Standings},
    settings::SwissPairingsSetting,
};

use cycle_map::GroupMap;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Swiss pairings are the "traditional" pairings system for Magic tournaments
pub struct SwissPairings {
    do_check_ins: bool,
    check_ins: HashSet<PlayerId>,
}

impl SwissPairings {
    /// Creates a new swiss pairings struct
    pub fn new() -> Self {
        SwissPairings {
            do_check_ins: false,
            check_ins: HashSet::new(),
        }
    }

    /// Marks a player as ready to play in their next round
    pub fn ready_player(&mut self, plyr: PlayerId) {
        self.check_ins.insert(plyr);
    }

    /// Marks a player as unready to play in their next round
    pub fn unready_player(&mut self, plyr: PlayerId) {
        self.check_ins.remove(&plyr);
    }

    /// Updates a single pairings setting
    pub fn update_setting(&mut self, setting: SwissPairingsSetting) {
        use SwissPairingsSetting::*;
        match setting {
            DoCheckIns(b) => {
                self.do_check_ins = b;
            }
        }
    }

    /// Calculates if the system can pair more rounds
    pub fn ready_to_pair(
        &self,
        match_size: usize,
        plyr_reg: &PlayerRegistry,
        rnd_reg: &RoundRegistry,
    ) -> bool {
        let count = plyr_reg.active_player_count();
        let mut digest = rnd_reg.active_round_count() == 0;
        digest &= count >= match_size;
        if self.do_check_ins {
            digest &= self.do_check_ins && count == self.check_ins.len();
        }
        digest
    }

    /// Attempts to create the next set of pairings.
    /// NOTE: This does not create new rounds, only pairings
    pub fn pair<S>(
        &mut self,
        alg: PairingAlgorithm,
        players: &PlayerRegistry,
        matches: &RoundRegistry,
        mut standings: Standings<S>,
        match_size: usize,
        repair_tol: u64,
    ) -> Option<Pairings>
    where
        S: Score,
    {
        if !self.ready_to_pair(match_size, players, matches) {
            return None;
        }
        let max_count = 100;
        let mut count = 0;
        let plyrs_and_scores: Vec<(PlayerId, u64)> = standings
            .scores
            .drain(0..)
            .filter_map(|(p, s)| {
                players
                    .get_player(&p.into())
                    .ok()?
                    .can_play()
                    .then(|| (p, s.primary_score() as u64))
            })
            .rev()
            .collect();
        let mut plyrs: Vec<PlayerId> = plyrs_and_scores.iter().map(|(p, _)| p).cloned().collect();
        let mut pairings = (alg.as_alg())(
            plyrs.drain(0..).collect(),
            &matches.opponents,
            match_size,
            repair_tol,
        );
        while count < max_count && pairings.rejected.len() != 0 {
            count += 1;
            let grouped_plyrs: GroupMap<_, _> = plyrs_and_scores.iter().cloned().collect();
            plyrs.extend(
                grouped_plyrs
                    .iter_right()
                    .map(|r| grouped_plyrs.get_left_iter(r).unwrap())
                    .flatten()
                    .cloned(),
            );
            let buffer = (alg.as_alg())(
                plyrs.drain(0..).collect(),
                &matches.opponents,
                match_size,
                repair_tol,
            );
            if buffer.rejected.len() < pairings.rejected.len() {
                pairings = buffer;
            }
        }
        Some(pairings)
    }
}
