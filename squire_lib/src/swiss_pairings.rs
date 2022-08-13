use crate::{
    identifiers::{PlayerId, PlayerIdentifier},
    pairings::Pairings,
    player::PlayerStatus,
    player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
    scoring::{Score, Standings},
    settings::SwissPairingsSetting,
};

use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Swiss pairings are the "traditional" pairings system for Magic tournaments
pub struct SwissPairings {
    players_per_match: u8,
    do_check_ins: bool,
    check_ins: HashSet<PlayerId>,
}

impl SwissPairings {
    /// Creates a new swiss pairings struct
    pub fn new(players_per_match: u8) -> Self {
        SwissPairings {
            players_per_match,
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
            MatchSize(s) => {
                self.players_per_match = s;
            }
            DoCheckIns(b) => {
                self.do_check_ins = b;
            }
        }
    }

    /// Calculates if the system can pair more rounds
    pub fn ready_to_pair(&self, plyr_reg: &PlayerRegistry, rnd_reg: &RoundRegistry) -> bool {
        let mut digest = true;
        if self.do_check_ins {
            digest &= plyr_reg.active_player_count() == self.check_ins.len();
        }
        digest &= rnd_reg.active_round_count() == 0;
        digest
    }

    /// Checks to see if a player can be apart of a potential pairing
    fn valid_pairing(&self, matches: &RoundRegistry, known: &[&PlayerId], new: &PlayerId) -> bool {
        if let Some(opps) = matches.opponents.get(new) {
            known.iter().any(|p| !opps.contains(p))
        } else {
            true
        }
    }

    /// Attempts to create the next set of pairings.
    /// NOTE: This does not create new rounds, only pairings
    pub fn pair<S>(
        &mut self,
        players: &PlayerRegistry,
        matches: &RoundRegistry,
        mut standings: Standings<S>,
    ) -> Option<Pairings>
    where
        S: Score,
    {
        if !self.ready_to_pair(players, matches) {
            return None;
        }
        let mut plyrs: Vec<PlayerId> = standings
            .scores
            .drain(0..)
            .filter_map(|(p, _)| {
                if players.get_player_status(&PlayerIdentifier::Id(p))? == PlayerStatus::Registered
                {
                    Some(p)
                } else {
                    None
                }
            })
            .rev()
            .collect();
        let mut digest = Pairings {
            paired: Vec::with_capacity(plyrs.len() / self.players_per_match as usize + 1),
            rejected: Vec::new(),
        };
        while plyrs.len() > self.players_per_match as usize {
            let mut index_buffer: Vec<usize> = Vec::with_capacity(self.players_per_match as usize);
            let mut id_buffer: Vec<&PlayerId> = Vec::with_capacity(self.players_per_match as usize);
            index_buffer.push(0);
            id_buffer.push(&plyrs[0]);
            for (i, _) in plyrs.iter().enumerate().skip(1) {
                if self.valid_pairing(matches, &id_buffer, &plyrs[i]) {
                    index_buffer.push(i);
                    id_buffer.push(&plyrs[i]);
                    if index_buffer.len() == self.players_per_match as usize {
                        break;
                    }
                }
            }
            if index_buffer.len() == self.players_per_match as usize {
                let mut pairing: Vec<PlayerId> =
                    Vec::with_capacity(self.players_per_match as usize);
                for i in index_buffer {
                    pairing.push(plyrs[i]);
                }
                digest.paired.push(pairing);
            } else {
                digest.rejected.push(plyrs.pop().unwrap());
            }
        }
        Some(digest)
    }
}
