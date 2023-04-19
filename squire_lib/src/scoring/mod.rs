use serde::{Deserialize, Serialize};

use crate::{
    identifiers::PlayerId,
    operations::{OpData, OpResult},
    players::PlayerRegistry,
    r64,
    rounds::RoundRegistry,
    settings::{
        CommonScoringSettingsTree, ScoringSetting, ScoringSettingsTree, ScoringStyleSetting,
        ScoringStyleSettingsTree,
    },
    tournament::TournamentPreset,
};

/// Contains the models for the standard score
pub mod standard_scoring;

pub use standard_scoring::{StandardScore, StandardScoring};

/// The trait the defines the interface for a score
pub trait Score
where
    Self: ToString,
{
    /// Returns the primary value of the score
    fn primary_score(&self) -> r64;
}

/// An ordered collection of scores
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Standings<S> {
    /// The player, score pairings
    pub scores: Vec<(PlayerId, S)>,
}

/// A scoring system that contain a style of calculating and ordering scores as well as some common
/// settings upon all scoring styles
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ScoringSystem {
    /// Settings common to call scoring systems
    #[serde(default)]
    pub common: CommonScoringSettingsTree,
    /// Settings of the active scoring system
    #[serde(default = "default_style")]
    pub style: ScoringStyle,
}

fn default_style() -> ScoringStyle {
    ScoringStyle::Standard(Default::default())
}

/// An enum that encodes all the possible scoring systems a tournament can have.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ScoringStyle {
    /// The tournament is using standard-style scoring
    Standard(StandardScoring),
}

impl ScoringStyle {
    /// Creates a new scoring style for a tournament preset
    pub fn new(_: TournamentPreset) -> Self {
        Self::Standard(Default::default())
    }

    /// Returns the current standings for all players
    pub fn get_standings(
        &self,
        plyrs: &PlayerRegistry,
        rnds: &RoundRegistry,
    ) -> Standings<StandardScore> {
        match self {
            ScoringStyle::Standard(style) => style.get_standings(plyrs, rnds),
        }
    }

    /// Returns a copy of the current settings
    pub fn settings(&self) -> ScoringStyleSettingsTree {
        match self {
            ScoringStyle::Standard(tree) => ScoringStyleSettingsTree::Standard(tree.settings()),
        }
    }

    /// Updates the current settings of the held scoring style
    pub fn update(&mut self, setting: ScoringStyleSetting) -> OpResult {
        match (self, setting) {
            (ScoringStyle::Standard(style), ScoringStyleSetting::Standard(setting)) => {
                style.update_setting(setting)
            }
        }
        Ok(OpData::Nothing)
    }
}

impl<S> Standings<S>
where
    S: Score,
{
    /// Creates a new, empty standings object
    pub fn new(scores: Vec<(PlayerId, S)>) -> Self {
        Standings { scores }
    }
}

impl ScoringSystem {
    /// Creates a new scoring system
    pub fn new(preset: TournamentPreset) -> Self {
        Self {
            common: CommonScoringSettingsTree::new(),
            style: ScoringStyle::new(preset),
        }
    }

    /// Returns a copy of the current settings
    pub fn settings(&self) -> ScoringSettingsTree {
        ScoringSettingsTree {
            common: self.common.clone(),
            style: self.style.settings(),
        }
    }

    /// Gets the current standings of all players
    pub fn get_standings(
        &self,
        player_reg: &PlayerRegistry,
        round_reg: &RoundRegistry,
    ) -> Standings<StandardScore> {
        self.style.get_standings(player_reg, round_reg)
    }

    /// Updates a given setting for the scoring system
    pub fn update_setting(&mut self, setting: ScoringSetting) -> OpResult {
        match setting {
            ScoringSetting::Common(setting) => self.common.update(setting),
            ScoringSetting::Style(setting) => self.style.update(setting),
        }
    }
}
