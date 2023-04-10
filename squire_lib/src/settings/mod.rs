use serde::{Deserialize, Serialize};

mod general;
mod pairing;
mod scoring;

pub use general::*;
pub use pairing::*;
pub use scoring::*;

/// An enum that encodes all the adjustable settings of a tournament
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum TournamentSetting {
    /// Adjusts a general tournament setting
    GeneralSetting(GeneralSetting),
    /// Adjusts a pairing system setting
    PairingSetting(PairingSetting),
    /// Adjusts a scoring system setting
    ScoringSetting(ScoringSetting),
}

/// A structure that contains 
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct TournamentSettingsTree {
    /// The set of tournament general settings
    general: GeneralSettingsTree,
    /// The set of tournament settings related to pairing
    pairing: PairingSettingsTree,
    /// The set of tournament settings related to scoring
    scoring: ScoringSettingsTree,
}

impl TournamentSettingsTree {
    /// Creates a new tournament settings tree with default values for all settings
    pub fn new() -> Self {
        Self {
            general: Default::default(),
            pairing: Default::default(),
            scoring: Default::default(),
        }
    }

    /// Updates the tournament settings tree, replacing one setting with the given setting
    pub fn update(&mut self, setting: TournamentSetting) {
        match setting {
            TournamentSetting::GeneralSetting(setting) => self.general.update(setting),
            TournamentSetting::PairingSetting(setting) => self.pairing.update(setting),
            TournamentSetting::ScoringSetting(setting) => self.scoring.update(setting),
        }
    }

    /// Returns an iterator over all the contained settings
    fn iter(&self) -> impl Iterator<Item = TournamentSetting> {
        self.general
            .iter()
            .map(Into::into)
            .chain(self.pairing.iter().map(Into::into))
            .chain(self.scoring.iter().map(Into::into))
    }

    /// Returns an iterator that yields settings from this tree what differ from the given tree
    pub fn diff(&self, other: &Self) -> impl Iterator<Item = TournamentSetting> {
        self.iter()
            .zip(other.iter())
            .filter_map(|(new, old)| (new == old).then_some(new))
    }
}
