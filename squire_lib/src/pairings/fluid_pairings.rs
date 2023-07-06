use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    identifiers::PlayerId,
    pairings::Pairings,
    players::PlayerRegistry,
    rounds::{RoundContext, RoundRegistry},
    settings::{FluidPairingSetting, FluidPairingSettingsTree, PairingCommonSettingsTree},
};

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
/// Fluid pairings are also known as a looking-for-game queue and are used for on-the-fly pairings
/// between players.
pub struct FluidPairings {
    #[serde(default)]
    settings: FluidPairingSettingsTree,
    check_ins: HashSet<PlayerId>,
    queue: Vec<PlayerId>,
}

impl FluidPairings {
    /// Creates a new fluid pairings struct
    pub fn new() -> Self {
        FluidPairings {
            settings: Default::default(),
            check_ins: HashSet::new(),
            queue: Vec::new(),
        }
    }

    /// Returns the current settings
    pub fn settings(&self) -> FluidPairingSettingsTree {
        FluidPairingSettingsTree {}
    }

    /// Marks a player as ready to play a game
    pub fn ready_player(&mut self, plyr: PlayerId) {
        if !self.queue.iter().any(|p| *p == plyr) {
            _ = self.check_ins.insert(plyr);
        }
    }

    /// Removes the player from the LFG queue
    pub fn unready_player(&mut self, plyr: PlayerId) {
        if self.check_ins.contains(&plyr) {
            _ = self.check_ins.remove(&plyr);
        } else if let Some(index) =
            self.queue
                .iter()
                .enumerate()
                .find_map(|(i, p)| if *p == plyr { Some(i) } else { None })
        {
            _ = self.queue.remove(index);
        }
    }

    /// Gets the round context for the system
    pub fn get_context(&self) -> RoundContext {
        RoundContext::Contextless
    }

    /// Updates a pairing setting
    pub fn update_setting(&mut self, setting: FluidPairingSetting) -> ! {
        //use FluidPairingsSetting::*;
        match setting {}
    }

    /// Calculates if a pairing is potentially possible
    pub fn ready_to_pair(&self, match_size: usize) -> bool {
        !self.check_ins.is_empty() && self.check_ins.len() + self.queue.len() >= match_size
    }

    /// Updates with incoming pairings.
    pub fn update(&mut self, pairings: &Pairings) {
        self.queue.extend(self.check_ins.drain());
        let plyrs: HashSet<_> = pairings.paired.iter().flatten().collect();
        self.queue.retain(|p| !plyrs.contains(p));
    }

    /// Attempts to pair all players in the queue.
    /// NOTE: This does not create any round, only pairings.
    pub fn pair(
        &self,
        common: &PairingCommonSettingsTree,
        _players: &PlayerRegistry,
        matches: &RoundRegistry,
    ) -> Option<Pairings> {
        let PairingCommonSettingsTree {
            match_size,
            repair_tolerance,
            algorithm,
        } = common;
        if !self.ready_to_pair(*match_size as usize) {
            return None;
        }
        let plyrs = self
            .queue
            .iter()
            .chain(self.check_ins.iter())
            .cloned()
            .collect();
        let mut digest = (algorithm.as_alg())(
            plyrs,
            &matches.opponents,
            *match_size as usize,
            *repair_tolerance,
        );
        drop(digest.rejected.drain(0..));
        Some(digest)
    }
}
