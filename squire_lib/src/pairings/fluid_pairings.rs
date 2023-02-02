use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    identifiers::PlayerId,
    pairings::{PairingAlgorithm, Pairings},
    players::PlayerRegistry,
    rounds::{RoundContext, RoundRegistry},
    settings::FluidPairingsSetting,
};

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
/// Fluid pairings are also known as a looking-for-game queue and are used for on-the-fly pairings
/// between players.
pub struct FluidPairings {
    check_ins: HashSet<PlayerId>,
    queue: Vec<PlayerId>,
}

impl FluidPairings {
    /// Creates a new fluid pairings struct
    pub fn new() -> Self {
        FluidPairings {
            check_ins: HashSet::new(),
            queue: Vec::new(),
        }
    }

    /// Marks a player as ready to play a game
    pub fn ready_player(&mut self, plyr: PlayerId) {
        if !self.queue.iter().any(|p| *p == plyr) {
            self.check_ins.insert(plyr);
        }
    }

    /// Removes the player from the LFG queue
    pub fn unready_player(&mut self, plyr: PlayerId) {
        if self.check_ins.contains(&plyr) {
            self.check_ins.remove(&plyr);
        } else if let Some(index) =
            self.queue
                .iter()
                .enumerate()
                .find_map(|(i, p)| if *p == plyr { Some(i) } else { None })
        {
            self.queue.remove(index);
        }
    }

    /// Gets the round context for the system
    pub fn get_context(&self) -> RoundContext {
        RoundContext::Contextless
    }

    /// Updates a pairing setting
    pub fn update_setting(&mut self, setting: FluidPairingsSetting) {
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
        alg: PairingAlgorithm,
        _players: &PlayerRegistry,
        matches: &RoundRegistry,
        match_size: usize,
        repair_tolerance: u64,
    ) -> Option<Pairings> {
        if !self.ready_to_pair(match_size) {
            return None;
        }
        let plyrs = self
            .queue
            .iter()
            .chain(self.check_ins.iter())
            .cloned()
            .collect();
        let mut digest = (alg.as_alg())(plyrs, &matches.opponents, match_size, repair_tolerance);
        digest.rejected.drain(0..);
        Some(digest)
    }
}
