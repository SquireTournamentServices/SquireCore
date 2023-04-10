use serde::{Deserialize, Serialize};

use crate::pairings::PairingAlgorithm;

/// An enum that encodes all the adjustable settings of all pairing systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum PairingSetting {
    /// Adjusts the number of players that will be in a match
    MatchSize(u8),
    /// Adjusts the number of pairs of players that have already played against each other that can be in a
    /// valid pairing
    RepairTolerance(u64),
    /// Adjusts the algorithm that will be used to pair players
    Algorithm(PairingAlgorithm),
    /// Settings for the swiss pairings system
    Swiss(SwissPairingSetting),
    /// Settings for the fluid pairings system
    Fluid(FluidPairingSetting),
}

/// A structure that holds a value for each pairing setting
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct PairingSettingsTree {
    match_size: u8,
    repair_tolerance: u64,
    algorithm: PairingAlgorithm,
    swiss: SwissPairingSettingsTree,
    fluid: FluidPairingSettingsTree,
}

impl PairingSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: PairingSetting) {
        match setting {
            PairingSetting::MatchSize(size) => self.match_size = size,
            PairingSetting::RepairTolerance(tol) => self.repair_tolerance = tol,
            PairingSetting::Algorithm(alg) => self.algorithm = alg,
            PairingSetting::Swiss(setting) => self.swiss.update(setting),
            PairingSetting::Fluid(setting) => self.fluid.update(setting),
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = PairingSetting> {
        vec![
            PairingSetting::MatchSize(self.match_size),
            PairingSetting::RepairTolerance(self.repair_tolerance),
            PairingSetting::Algorithm(self.algorithm),
        ]
        .into_iter()
        .chain(self.swiss.iter().map(Into::into))
        .chain(self.fluid.iter().map(Into::into))
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
    do_checkins: bool,
}

impl SwissPairingSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: SwissPairingSetting) {
        match setting {
            SwissPairingSetting::DoCheckIns(b) => self.do_checkins = b,
        }
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
    pub fn update(&mut self, setting: FluidPairingSetting) {
        #[allow(irrefutable_let_patterns)]
        match setting {}
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = FluidPairingSetting> {
        vec![].into_iter()
    }
}
