use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::TournamentError,
    fluid_pairings::FluidPairings,
    identifiers::PlayerId,
    operations::{OpData, OpResult},
    player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
    scoring::{Score, Standings},
    settings::PairingSetting,
    swiss_pairings::SwissPairings,
    tournament::TournamentPreset,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct for communicating new pairings information
pub struct Pairings {
    /// The players that are paired and their groupings
    pub paired: Vec<Vec<PlayerId>>,
    /// The players that aren't paired
    pub rejected: Vec<PlayerId>,
}

/// Encodes what algorithm will be used to pair players
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingAlgorithm {
    /// This variant coresponds to the `greedy_pairings` function
    #[default]
    Greedy,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An enum that encodes all the possible pairing systems a tournament can have.
pub struct PairingSystem {
    /// The number of players that will be in new Rounds
    pub match_size: u8,
    /// The number of pairs of players that have already played against each other that can be in a
    /// valid pairing
    pub repair_tolerance: u64,
    /// The algorithm used to pair players
    pub alg: PairingAlgorithm,
    /// The style of pairings that is used
    pub style: PairingStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An enum that encodes all the possible pairing systems a tournament can have.
pub enum PairingStyle {
    /// The tournament has a swiss pairing system
    Swiss(SwissPairings),
    /// The tournament has a fluid pairing system
    Fluid(FluidPairings),
}

impl PairingSystem {
    /// Creates a new pairing system
    pub fn new(preset: TournamentPreset) -> Self {
        use TournamentPreset::*;
        let style: PairingStyle = match preset {
            Swiss => SwissPairings::new().into(),
            Fluid => FluidPairings::new().into(),
        };
        PairingSystem {
            match_size: 2,
            repair_tolerance: 0,
            alg: PairingAlgorithm::Greedy,
            style,
        }
    }
    /// Marks a player as ready to play in their next round
    pub fn ready_player(&mut self, id: PlayerId) {
        use PairingStyle::*;
        match &mut self.style {
            Swiss(sys) => sys.ready_player(id),
            Fluid(sys) => sys.ready_player(id),
        }
    }

    /// Removes the player from the LFG queue
    pub fn unready_player(&mut self, id: PlayerId) {
        use PairingStyle::*;
        match &mut self.style {
            Swiss(sys) => sys.unready_player(id),
            Fluid(sys) => sys.unready_player(id),
        }
    }

    /// Calculates if the pairing system is able to create a set of pairings
    pub fn ready_to_pair(&self, plyr_reg: &PlayerRegistry, rnd_reg: &RoundRegistry) -> bool {
        use PairingStyle::*;
        match &self.style {
            Swiss(sys) => sys.ready_to_pair(self.match_size as usize, plyr_reg, rnd_reg),
            Fluid(sys) => sys.ready_to_pair(self.match_size as usize),
        }
    }

    /// Attempts to create the next set of pairings
    pub fn pair<S>(
        &mut self,
        plyr_reg: &PlayerRegistry,
        rnd_reg: &RoundRegistry,
        standings: Standings<S>,
    ) -> Option<Pairings>
    where
        S: Score,
    {
        use PairingStyle::*;
        match &mut self.style {
            Swiss(sys) => sys.pair(
                self.alg,
                plyr_reg,
                rnd_reg,
                standings,
                self.match_size as usize,
                self.repair_tolerance,
            ),
            Fluid(sys) => sys.pair(
                self.alg,
                plyr_reg,
                rnd_reg,
                self.match_size as usize,
                self.repair_tolerance,
            ),
        }
    }

    /// Updates a setting of the pairing system or its pairing style
    pub fn update_setting(&mut self, setting: PairingSetting) -> OpResult {
        use PairingSetting::*;
        match setting {
            MatchSize(size) => {
                self.match_size = size;
            }
            RepairTolerance(tol) => {
                self.repair_tolerance = tol;
            }
            Algorithm(alg) => {
                self.alg = alg;
            }
            Swiss(s) => {
                if let PairingStyle::Swiss(sys) = &mut self.style {
                    sys.update_setting(s);
                } else {
                    return Err(TournamentError::IncompatiblePairingSystem);
                }
            }
            Fluid(s) => {
                if let PairingStyle::Fluid(sys) = &mut self.style {
                    sys.update_setting(s);
                } else {
                    return Err(TournamentError::IncompatiblePairingSystem);
                }
            }
        }
        Ok(OpData::Nothing)
    }
}

impl PairingAlgorithm {
    /// Returns a closure that contains the function that coresponds to the algorithm.
    pub fn as_alg(
        &self,
    ) -> impl FnOnce(Vec<PlayerId>, &HashMap<PlayerId, HashSet<PlayerId>>, usize, u64) -> Pairings
    {
        use PairingAlgorithm::*;
        match self {
            Greedy => greedy_pairings,
        }
    }
}

// TODO: PLEASE provide a better description
/// A simple, greedy algorithm that attempts to pair the first players first
pub fn greedy_pairings(
    mut plyrs: Vec<PlayerId>,
    opps: &HashMap<PlayerId, HashSet<PlayerId>>,
    match_size: usize,
    repair_tol: u64,
) -> Pairings {
    let mut digest = Pairings {
        paired: Vec::with_capacity(plyrs.len() / match_size + 1),
        rejected: Vec::new(),
    };
    while plyrs.len() >= match_size {
        let mut index_buffer: Vec<usize> = Vec::with_capacity(match_size);
        let mut id_buffer: Vec<PlayerId> = Vec::with_capacity(match_size);
        index_buffer.push(0);
        id_buffer.push(plyrs[0]);
        for (i, _) in plyrs.iter().enumerate().skip(1) {
            if valid_pairing(opps, &id_buffer, &plyrs[i], repair_tol) {
                index_buffer.push(i);
                id_buffer.push(plyrs[i]);
                if index_buffer.len() == match_size {
                    break;
                }
            }
        }
        if index_buffer.len() == match_size {
            let mut pairing: Vec<PlayerId> = Vec::with_capacity(match_size);
            for (count, i) in index_buffer.iter().enumerate() {
                let id = plyrs.remove(i - count);
                pairing.push(id);
            }
            digest.paired.push(pairing);
        } else {
            digest.rejected.push(plyrs.pop().unwrap());
        }
    }
    digest.rejected.extend_from_slice(&plyrs);
    digest
}

/// Checks to see if a player can be apart of a potential pairing
fn valid_pairing(
    past_opponents: &HashMap<PlayerId, HashSet<PlayerId>>,
    known: &[PlayerId],
    new: &PlayerId,
    repair_tol: u64,
) -> bool {
    if let Some(opps) = past_opponents.get(new) {
        known.iter().filter(|p| opps.contains(p)).count() as u64 <= repair_tol
    } else {
        true
    }
}

impl Display for PairingAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PairingAlgorithm::*;
        match self {
            Greedy => write!(f, "Greedy"),
        }
    }
}

impl From<SwissPairings> for PairingStyle {
    fn from(other: SwissPairings) -> Self {
        Self::Swiss(other)
    }
}

impl From<FluidPairings> for PairingStyle {
    fn from(other: FluidPairings) -> Self {
        Self::Fluid(other)
    }
}
