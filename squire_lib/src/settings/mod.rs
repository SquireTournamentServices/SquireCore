#![allow(clippy::module_name_repetitions)]
use serde::{Deserialize, Serialize};

mod general;
mod pairing;
mod scoring;

pub use general::*;
pub use pairing::*;
pub use scoring::*;

use crate::{operations::OpResult, tournament::TournamentPreset};

// TODO: These dyn iterators should be replaced with `impl Iterator` once Rust issue #91611
// (https://github.com/rust-lang/rust/issues/91611) return_position_impl_trait_in_trait is
// completed.
/// A trait that encapsulates the methods needed by every settings tree
pub trait SettingsTree: Default {
    /// The setting type used by this tree
    type Setting: 'static + PartialEq;

    /// Creates a new, default settings tree
    fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    fn update(&mut self, setting: Self::Setting) -> OpResult;

    /// Returns an iterator over all the contained settings
    fn iter(&self) -> Box<dyn Iterator<Item = Self::Setting>>;

    /// Returns an iterator over this tree that yields all the settings that differ between the two
    fn diff(&self, other: &Self) -> Box<dyn Iterator<Item = Self::Setting>> {
        Box::new(
            self.iter()
                .zip(other.iter())
                .filter_map(|(new, old)| (new != old).then_some(new)),
        )
    }
}

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
    pub general: GeneralSettingsTree,
    /// The set of tournament settings related to pairing
    pub pairing: PairingSettingsTree,
    /// The set of tournament settings related to scoring
    pub scoring: ScoringSettingsTree,
}

impl Default for TournamentSettingsTree {
    fn default() -> Self {
        Self {
            general: GeneralSettingsTree::default(),
            pairing: PairingSettingsTree::with_preset(TournamentPreset::Swiss),
            scoring: ScoringSettingsTree::with_preset(TournamentPreset::Swiss),
        }
    }
}

impl TournamentSettingsTree {
    /// Creates a new tournament settings tree with default values for all settings
    #[must_use]
    pub fn with_preset(preset: TournamentPreset) -> Self {
        Self {
            general: GeneralSettingsTree::default(),
            pairing: PairingSettingsTree::with_preset(preset),
            scoring: ScoringSettingsTree::with_preset(preset),
        }
    }
}

impl SettingsTree for TournamentSettingsTree {
    type Setting = TournamentSetting;

    fn update(&mut self, setting: Self::Setting) -> OpResult {
        match setting {
            TournamentSetting::GeneralSetting(setting) => self.general.update(setting),
            TournamentSetting::PairingSetting(setting) => self.pairing.update(setting),
            TournamentSetting::ScoringSetting(setting) => self.scoring.update(setting),
        }
    }

    /// Returns an iterator over all the contained settings
    fn iter(&self) -> Box<dyn Iterator<Item = Self::Setting>> {
        Box::new(
            self.general
                .iter()
                .map(Into::into)
                .chain(self.pairing.iter().map(Into::into))
                .chain(self.scoring.iter().map(Into::into)),
        )
    }
}
