use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::TournamentError,
    identifiers::PlayerId,
    operations::{OpData, OpResult},
    players::PlayerRegistry,
    rounds::RoundRegistry,
    scoring::{Score, Standings},
    settings::PairingSetting,
    tournament::TournamentPreset,
};

/// The fluid pairing sytle
pub mod fluid_pairings;
/// The swiss pairing sytle
pub mod swiss_pairings;

/// The branching pairings module
pub mod branching;
/// The greedy pairings module
pub mod greedy;
/// The rotary pairings module
pub mod rotary;

pub use branching::branching_pairings;
pub use fluid_pairings::FluidPairings;
pub use greedy::greedy_pairings;
pub use rotary::rotary_pairings;
pub use swiss_pairings::SwissPairings;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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
    /// This variant corresponds to the `greedy_pairings` function
    Greedy,
    /// This variant corresponds to the `branching_pairings` function
    #[default]
    Branching,
    /// This variant corresponds to the `rotary_pairings` function
    Rotary,
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

impl Pairings {
    /// Creates empty pairings
    pub fn new() -> Self {
        Self {
            paired: Vec::new(),
            rejected: Vec::new(),
        }
    }

    /// Calculates if the pairings are all valid
    pub fn is_valid(&self, opps: &HashMap<PlayerId, HashSet<PlayerId>>, repair_tol: u64) -> bool {
        self.paired
            .iter()
            .find(|p| count_opps(p, opps) > repair_tol)
            .is_none()
    }
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
            alg: PairingAlgorithm::Branching,
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
            Branching => branching_pairings,
            Rotary => rotary_pairings,
        }
    }
}

impl Display for PairingAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PairingAlgorithm::*;
        match self {
            Greedy => write!(f, "Greedy"),
            Branching => write!(f, "Branching"),
            Rotary => write!(f, "Rotary"),
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

/// Calculates the number of repeat opponents there are in a set of players
pub fn count_opps(plyrs: &[PlayerId], opps: &HashMap<PlayerId, HashSet<PlayerId>>) -> u64 {
    let mut digest = 0;
    let iter = plyrs.iter();
    for p in iter.clone() {
        let inner = iter.clone();
        for p_inner in inner {
            digest += opps.get(p).map(|o| o.contains(p_inner) as u64).unwrap_or(0);
        }
    }
    digest
}
