use crate::{
    tournament::PairingSystem,
    identifiers::PlayerId, pairings::Pairings, player_registry::PlayerRegistry,
    round_registry::RoundRegistry, settings::FluidPairingsSetting,
};

use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Fluid pairings are also known as a looking-for-game queue and are used for on-the-fly pairings
/// between players.
pub struct FluidPairings {
    players_per_match: u8,
    check_ins: HashSet<PlayerId>,
    queue: Vec<PlayerId>,
}

impl Into<PairingSystem> for FluidPairings {
    fn into(self) -> PairingSystem {
        PairingSystem::Fluid(self)
    }
}

impl FluidPairings {
    /// Creates a new fluid pairings struct
    pub fn new(players_per_match: u8) -> Self {
        FluidPairings {
            players_per_match,
            check_ins: HashSet::new(),
            queue: Vec::new(),
        }
    }

    /// Marks a player as ready to play a game
    pub fn ready_player(&mut self, plyr: PlayerId) {
        self.check_ins.insert(plyr);
    }

    /// Removes the player from the LFG queue
    pub fn unready_player(&mut self, plyr: PlayerId) {
        self.check_ins.remove(&plyr);
    }

    /// Updates a pairing setting
    pub fn update_setting(&mut self, setting: FluidPairingsSetting) {
        use FluidPairingsSetting::*;
        match setting {
            MatchSize(s) => {
                self.players_per_match = s;
            }
        }
    }

    /// Calculates if a pairing is potentially possible
    pub fn ready_to_pair(&self) -> bool {
        self.check_ins.len() + self.queue.len() >= self.players_per_match as usize
    }

    /// Checks to see if a player can be apart of a potential pairing
    fn valid_pairing(&self, matches: &RoundRegistry, known: &[&PlayerId], new: &PlayerId) -> bool {
        if let Some(opps) = matches.opponents.get(new) {
            known.iter().any(|p| !opps.contains(p))
        } else {
            true
        }
    }

    /// Attempts to pair all players in the queue.
    /// NOTE: This does not create any round, only pairings.
    pub fn pair(&mut self, _players: &PlayerRegistry, matches: &RoundRegistry) -> Option<Pairings> {
        if !self.ready_to_pair() {
            return None;
        }
        let mut plyrs: Vec<PlayerId> = self.check_ins.drain().collect();
        plyrs.extend(self.queue.drain(0..));
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
                self.queue.push(plyrs.pop().unwrap());
            }
        }
        Some(digest)
    }
}
