use serde::{Deserialize, Serialize};

use crate::{
    identifiers::PlayerId,
    operations::{OpData, OpResult},
    players::PlayerRegistry,
    r64,
    rounds::RoundRegistry,
    settings::ScoringSetting,
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

/// A scoring system that contain a style of calcualting and ordering scores as well as some common
/// settings upon all scoring styles
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ScoringSystem {
    style: ScoringStyle,
}

/// An enum that encodes all the possible scoring systems a tournament can have.
/// (So many, much wow)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ScoringStyle {
    /// The tournament has a standard scoring system
    Standard(StandardScoring),
}

/// An ordered collection of scores
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Standings<S> {
    /// The player, score pairings
    pub scores: Vec<(PlayerId, S)>,
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
    pub fn new(_: TournamentPreset) -> Self {
        Self {
            style: ScoringStyle::Standard(StandardScoring::new()),
        }
    }
    /// Gets the current standings of all players
    pub fn get_standings(
        &self,
        player_reg: &PlayerRegistry,
        round_reg: &RoundRegistry,
    ) -> Standings<StandardScore> {
        match &self.style {
            ScoringStyle::Standard(s) => s.get_standings(player_reg, round_reg),
        }
    }

    /// Updates a given setting for the scoring system
    pub fn update_setting(&mut self, setting: ScoringSetting) -> OpResult {
        match (&mut self.style, setting) {
            (ScoringStyle::Standard(style), ScoringSetting::Standard(setting)) => {
                style.update_setting(setting);
                Ok(OpData::Nothing)
            }
        }
    }
}
