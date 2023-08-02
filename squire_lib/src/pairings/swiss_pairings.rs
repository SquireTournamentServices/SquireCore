use std::collections::HashSet;

use cycle_map::GroupMap;
use serde::{Deserialize, Serialize};

use crate::{
    identifiers::PlayerId,
    operations::OpResult,
    pairings::Pairings,
    players::PlayerRegistry,
    r64,
    rounds::{RoundContext, RoundRegistry},
    scoring::{Score, Standings},
    settings::{
        PairingCommonSettingsTree, SettingsTree, SwissPairingSetting, SwissPairingSettingsTree,
    },
};

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash, PartialEq, Eq)]
/// The round context for swiss rounds
pub struct SwissContext {
    swiss_round_number: u8,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
/// Swiss pairings are the "traditional" pairings system for Magic tournaments
pub struct SwissPairings {
    #[serde(default)]
    settings: SwissPairingSettingsTree,
    check_ins: HashSet<PlayerId>,
    #[serde(default)]
    swiss_round_number: u8,
}

impl SwissPairings {
    /// Creates a new swiss pairings struct
    pub fn new() -> Self {
        SwissPairings {
            settings: Default::default(),
            check_ins: HashSet::new(),
            swiss_round_number: 0,
        }
    }

    /// Returns if this pairing method requires checkins
    pub fn settings(&self) -> SwissPairingSettingsTree {
        self.settings.clone()
    }

    /// Returns if this pairing method requires checkins
    pub fn do_check_ins(&self) -> bool {
        self.settings.do_checkins
    }

    /// Marks a player as ready to play in their next round
    pub fn ready_player(&mut self, plyr: PlayerId) {
        _ = self.check_ins.insert(plyr);
    }

    /// Marks a player as unready to play in their next round
    pub fn unready_player(&mut self, plyr: PlayerId) {
        _ = self.check_ins.remove(&plyr);
    }

    /// Updates a single pairings setting
    pub fn update_setting(&mut self, setting: SwissPairingSetting) -> OpResult {
        self.settings.update(setting)
    }

    /// Calculates if the system can pair more rounds
    pub fn ready_to_pair(
        &self,
        match_size: usize,
        plyr_reg: &PlayerRegistry,
        rnd_reg: &RoundRegistry,
    ) -> bool {
        let SwissPairingSettingsTree { do_checkins } = self.settings;
        let count = plyr_reg.active_player_count();
        let mut digest = rnd_reg.active_round_count() == 0;
        digest &= count >= match_size;
        if do_checkins {
            digest &= do_checkins && count == self.check_ins.len();
        }
        digest
    }

    /// Gets the round context for the system
    pub fn get_context(&self) -> RoundContext {
        RoundContext::Swiss(SwissContext {
            swiss_round_number: self.swiss_round_number,
        })
    }

    /// Updates with incoming pairings.
    pub fn update(&mut self, pairings: &Pairings) {
        self.swiss_round_number = self.swiss_round_number.saturating_add(1); // TODO determine necessary size for swiss_round_number
        for p in pairings
            .paired
            .iter()
            .flatten()
            .chain(pairings.rejected.iter())
        {
            _ = self.check_ins.remove(p);
        }
    }

    /// Attempts to create the next set of pairings.
    /// NOTE: This does not create new rounds, only pairings
    pub fn pair<S>(
        &self,
        common: &PairingCommonSettingsTree,
        players: &PlayerRegistry,
        matches: &RoundRegistry,
        mut standings: Standings<S>,
    ) -> Option<Pairings>
    where
        S: Score,
    {
        let PairingCommonSettingsTree {
            match_size,
            repair_tolerance,
            algorithm,
        } = common;
        if !self.ready_to_pair(*match_size as usize, players, matches) {
            return None;
        }
        let plyrs_and_scores: Vec<(PlayerId, r64)> = standings
            .scores
            .drain(0..)
            .filter_map(|(p, s)| {
                players
                    .get_player(&p)
                    .ok()?
                    .can_play()
                    .then(|| (p, s.primary_score()))
            })
            .rev()
            .collect();
        let mut plyrs: Vec<PlayerId> = plyrs_and_scores.iter().map(|(p, _)| p).cloned().collect();
        let mut pairings = (*algorithm).as_alg()(
            std::mem::take(&mut plyrs),
            &matches.opponents,
            *match_size as usize,
            *repair_tolerance,
        );

        for _ in 0..100 {
            if pairings.rejected.is_empty() {
                break;
            }
            let grouped_plyrs: GroupMap<_, _> = plyrs_and_scores.iter().cloned().collect();
            plyrs.extend(grouped_plyrs.iter().filter_map(|(plyr, _)| plyr).cloned());
            let buffer = (*algorithm).as_alg()(
                std::mem::take(&mut plyrs),
                &matches.opponents,
                *match_size as usize,
                *repair_tolerance,
            );
            if buffer.rejected.len() < pairings.rejected.len() {
                pairings = buffer;
            }
        }
        Some(pairings)
    }
}
