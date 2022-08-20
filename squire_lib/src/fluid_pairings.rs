use crate::{
    identifiers::PlayerId,
    pairings::{PairingAlgorithm, Pairings},
    player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
    settings::FluidPairingsSetting,
};

use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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
        if self.queue.iter().find(|p| **p == plyr).is_none() {
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

    /// Updates a pairing setting
    pub fn update_setting(&mut self, setting: FluidPairingsSetting) {
        //use FluidPairingsSetting::*;
        match setting {}
    }

    /// Calculates if a pairing is potentially possible
    pub fn ready_to_pair(&self, match_size: usize) -> bool {
        !self.check_ins.is_empty() && self.check_ins.len() + self.queue.len() >= match_size
    }

    /// Attempts to pair all players in the queue.
    /// NOTE: This does not create any round, only pairings.
    pub fn pair(
        &mut self,
        alg: PairingAlgorithm,
        _players: &PlayerRegistry,
        matches: &RoundRegistry,
        match_size: usize,
        repair_tolerance: u64,
    ) -> Option<Pairings> {
        if !self.ready_to_pair(match_size) {
            return None;
        }
        let mut plyrs: Vec<PlayerId> = self.queue.drain(0..).rev().collect();
        plyrs.extend(self.check_ins.drain());
        let mut digest = (alg.as_alg())(plyrs, &matches.opponents, match_size, repair_tolerance);
        self.queue.extend(digest.rejected.drain(0..).rev());
        Some(digest)
    }
}
