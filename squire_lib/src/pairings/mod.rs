use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::TournamentError,
    identifiers::{PlayerId, RoundId},
    operations::OpResult,
    players::PlayerRegistry,
    rounds::{Round, RoundContext, RoundRegistry},
    scoring::{Score, Standings},
    settings::{
        PairingCommonSettingsTree, PairingSetting, PairingSettingsTree, PairingStyleSetting,
        PairingStyleSettingsTree,
    },
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

/// A struct for communicating new pairings information
#[derive(Serialize, Deserialize, Debug, Default, Hash, Clone, PartialEq, Eq)]
pub struct Pairings {
    /// The players that are paired and their groupings
    pub paired: Vec<Vec<PlayerId>>,
    /// The players that aren't paired
    pub rejected: Vec<PlayerId>,
}

impl Pairings {
    pub(crate) fn get_ids(&self, salt: DateTime<Utc>) -> Vec<RoundId> {
        self.paired
            .iter()
            .map(|plyrs| Round::create_id(salt, plyrs))
            .chain(self.rejected.iter().map(|p| Round::create_id(salt, &[*p])))
            .collect()
    }

    pub(crate) fn swap_player_ids(&mut self, old: PlayerId, new: PlayerId) {
        self.paired
            .iter_mut()
            .flatten()
            .filter(|p| **p == old)
            .for_each(|p| {
                *p = new;
            });
        self.rejected
            .iter_mut()
            .filter(|p| **p == old)
            .for_each(|p| {
                *p = new;
            });
    }
}

/// Encodes what algorithm will be used to pair players
#[derive(Serialize, Deserialize, Default, Debug, Clone, Hash, Copy, PartialEq, Eq)]
pub enum PairingAlgorithm {
    /// This variant corresponds to the `greedy_pairings` function
    Greedy,
    /// This variant corresponds to the `branching_pairings` function
    #[default]
    Branching,
    /// This variant corresponds to the `rotary_pairings` function
    Rotary,
}

/// An enum that encodes all the possible pairing systems a tournament can have.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PairingSystem {
    /// Settings common to all pairing styles
    pub common: PairingCommonSettingsTree,
    /// The style of pairings that is used
    pub style: PairingStyle,
}

/// An enum that encodes all the possible pairing systems a tournament can have.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

    /// Calculates the length of the paired and rejected players
    pub fn len(&self) -> usize {
        self.paired.len() + self.rejected.len()
    }

    /// Calculates if the pairings are empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Calculates if the pairings are all valid
    pub fn is_valid(&self, opps: &HashMap<PlayerId, HashSet<PlayerId>>, repair_tol: u64) -> bool {
        !self.paired.iter().any(|p| count_opps(p, opps) > repair_tol)
    }
}

impl PairingSystem {
    /// Creates a new pairing system
    pub fn new(preset: TournamentPreset) -> Self {
        use TournamentPreset::*;
        let common = PairingCommonSettingsTree {
            match_size: 2,
            repair_tolerance: 0,
            algorithm: PairingAlgorithm::Branching,
        };
        let style: PairingStyle = match preset {
            Swiss => SwissPairings::new().into(),
            Fluid => FluidPairings::new().into(),
        };
        PairingSystem { common, style }
    }

    /// Returns a copy of the current set of settings
    pub fn settings(&self) -> PairingSettingsTree {
        PairingSettingsTree {
            common: self.common.clone(),
            style: self.style.settings(),
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
            Swiss(sys) => sys.ready_to_pair(self.common.match_size as usize, plyr_reg, rnd_reg),
            Fluid(sys) => sys.ready_to_pair(self.common.match_size as usize),
        }
    }

    /// Gets the round context for the system
    pub fn get_context(&self) -> RoundContext {
        use PairingStyle::*;
        match &self.style {
            Swiss(sys) => sys.get_context(),
            Fluid(sys) => sys.get_context(),
        }
    }

    /// Updates the inner pairing style with incoming pairings.
    pub fn update(&mut self, pairings: &Pairings) {
        use PairingStyle::*;
        match &mut self.style {
            Swiss(sys) => sys.update(pairings),
            Fluid(sys) => sys.update(pairings),
        }
    }

    /// Attempts to create the next set of pairings
    pub fn pair<S>(
        &self,
        plyr_reg: &PlayerRegistry,
        rnd_reg: &RoundRegistry,
        standings: Standings<S>,
    ) -> Option<Pairings>
    where
        S: Score,
    {
        use PairingStyle::*;
        match &self.style {
            Swiss(sys) => sys.pair(&self.common, plyr_reg, rnd_reg, standings),
            Fluid(sys) => sys.pair(&self.common, plyr_reg, rnd_reg),
        }
    }

    /// Updates a setting of the pairing system or its pairing style
    pub fn update_setting(&mut self, setting: PairingSetting) -> OpResult {
        use PairingSetting::*;
        match setting {
<<<<<<< HEAD
            MatchSize(size) => {
                if size == 0 {
                    return Err(TournamentError::InvalidMatchSize)
                }
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
=======
            Common(setting) => self.common.update(setting),
            Style(s) => self.style.update(s),
        }
    }
}

impl PairingStyle {
    /// Creates empty pairings
    pub fn new(preset: TournamentPreset) -> Self {
        match preset {
            TournamentPreset::Swiss => Self::Swiss(Default::default()),
            TournamentPreset::Fluid => Self::Fluid(Default::default()),
        }
    }

    /// Returns a copy of the current set of settings
    pub fn settings(&self) -> PairingStyleSettingsTree {
        match self {
            PairingStyle::Swiss(style) => PairingStyleSettingsTree::Swiss(style.settings()),
            PairingStyle::Fluid(style) => PairingStyleSettingsTree::Fluid(style.settings()),
        }
    }

    /// Attempts to update the settings of the held pairing style
    pub fn update(&mut self, setting: PairingStyleSetting) -> OpResult {
        match (self, setting) {
            (PairingStyle::Swiss(style), PairingStyleSetting::Swiss(setting)) => {
                style.update_setting(setting)
            }
            (PairingStyle::Fluid(style), PairingStyleSetting::Fluid(setting)) => {
                style.update_setting(setting)
            }
            _ => Err(TournamentError::IncompatiblePairingSystem),
>>>>>>> 4fdb12a (Finished settings refactor.)
        }
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
