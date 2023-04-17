use serde::{Deserialize, Serialize};

use crate::{
    error::TournamentError,
    operations::{OpData, OpResult},
    pairings::PairingAlgorithm,
    tournament::TournamentPreset,
};

/// An enum that encodes all the adjustable settings of all pairing systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum PairingSetting {
    /// Setting common among all pairing styles
    Common(CommonPairingSetting),
    /// Settings for the pairing style
    Style(PairingStyleSetting),
}

/// Settings that are common among all pairing systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum CommonPairingSetting {
    /// Adjusts the number of players that will be in a match
    MatchSize(u8),
    /// Adjusts the number of pairs of players that have already played against each other that can be in a
    /// valid pairing
    RepairTolerance(u64),
    /// Adjusts the algorithm that will be used to pair players
    Algorithm(PairingAlgorithm),
}

/// Settings for a given pairing style
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum PairingStyleSetting {
    /// Settings for the swiss-style of pairings
    Swiss(SwissPairingSetting),
    /// Settings for the fluid-style of pairings
    Fluid(FluidPairingSetting),
}

/// A structure that holds a value for each pairing setting
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct PairingSettingsTree {
    /// Settings used by all pairing methods
    pub common: PairingCommonSettingsTree,
    /// The settings for the style of pairings being used
    pub style: PairingStyleSettingsTree,
}

/// A enum that holds settings for the active pairing sytle
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum PairingStyleSettingsTree {
    /// The set of settings for swiss-style pairings
    Swiss(SwissPairingSettingsTree),
    /// The set of settings for fluid-style pairings
    Fluid(FluidPairingSettingsTree),
}

/// A structure that holds settings common to all pairing systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct PairingCommonSettingsTree {
    /// The number of players that will be in new Rounds
    pub match_size: u8,
    /// The number of pairs of players that have already played against each other that can be in a
    /// valid pairing
    pub repair_tolerance: u64,
    /// The algorithm used to pair players
    pub algorithm: PairingAlgorithm,
}

impl PairingSettingsTree {
    /// Creates a new, default settings tree
    pub fn new(preset: TournamentPreset) -> Self {
        Self {
            common: Default::default(),
            style: PairingStyleSettingsTree::new(preset),
        }
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: PairingSetting) -> OpResult {
        match setting {
            PairingSetting::Common(setting) => self.common.update(setting),
            PairingSetting::Style(setting) => self.style.update(setting),
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = PairingSetting> {
        self.common
            .iter()
            .map(Into::into)
            .chain(self.style.iter().map(Into::into))
    }
}

impl PairingStyleSettingsTree {
    /// Creates a new, default settings tree
    pub fn new(preset: TournamentPreset) -> Self {
        match preset {
            TournamentPreset::Swiss => Self::Swiss(Default::default()),
            TournamentPreset::Fluid => Self::Fluid(Default::default()),
        }
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: PairingStyleSetting) -> OpResult {
        match (self, setting) {
            (PairingStyleSettingsTree::Swiss(style), PairingStyleSetting::Swiss(setting)) => {
                style.update(setting)
            }
            (PairingStyleSettingsTree::Fluid(style), PairingStyleSetting::Fluid(setting)) => {
                style.update(setting)
            }
            _ => Err(TournamentError::IncompatiblePairingSystem),
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = PairingSetting> {
        let digest: Box<dyn Iterator<Item = PairingSetting>> = match self {
            PairingStyleSettingsTree::Swiss(style) => Box::new(style.iter().map(Into::into)),
            PairingStyleSettingsTree::Fluid(style) => Box::new(style.iter().map(Into::into)),
        };
        digest
    }
}

impl PairingCommonSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: CommonPairingSetting) -> OpResult {
        match setting {
            CommonPairingSetting::MatchSize(size) => {
                if size == 0 {
                    return Err(TournamentError::InvalidMatchSize);
                }
                self.match_size = size;
            }
            CommonPairingSetting::RepairTolerance(tol) => self.repair_tolerance = tol,
            CommonPairingSetting::Algorithm(alg) => self.algorithm = alg,
        }
        Ok(OpData::Nothing)
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = PairingSetting> {
        vec![
            CommonPairingSetting::MatchSize(self.match_size),
            CommonPairingSetting::RepairTolerance(self.repair_tolerance),
            CommonPairingSetting::Algorithm(self.algorithm),
        ]
        .into_iter()
        .map(Into::into)
    }
}

/// An enum that encodes all the adjustable settings of swiss pairing systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum SwissPairingSetting {
    /// Whether or not player need to check in before a round is paired
    DoCheckIns(bool),
}

/// A structure that holds a value for each pairing setting
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct SwissPairingSettingsTree {
    /// Whether or not checkins need to performed before pairings can be created
    pub do_checkins: bool,
}

impl SwissPairingSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: SwissPairingSetting) -> OpResult {
        match setting {
            SwissPairingSetting::DoCheckIns(b) => self.do_checkins = b,
        }
        Ok(OpData::Nothing)
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = SwissPairingSetting> {
        vec![SwissPairingSetting::DoCheckIns(self.do_checkins)].into_iter()
    }
}

/// An enum that encodes all the adjustable settings of fluid pairing systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum FluidPairingSetting {}

/// A structure that holds a value for each pairing setting
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct FluidPairingSettingsTree {}

impl FluidPairingSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: FluidPairingSetting) -> ! {
        match setting {}
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = FluidPairingSetting> {
        vec![].into_iter()
    }
}
