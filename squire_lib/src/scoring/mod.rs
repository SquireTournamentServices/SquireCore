use serde::{Deserialize, Serialize};

use crate::{identifiers::PlayerId, players::PlayerRegistry, r64, rounds::RoundRegistry};

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An enum that encodes all the possible scoring systems a tournament can have.
/// (So many, much wow)
pub enum ScoringSystem {
    /// The tournament has a standard scoring system
    Standard(StandardScoring),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// An ordered collection of scores
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
    /// Gets the current standings of all players
    pub fn get_standings(
        &self,
        player_reg: &PlayerRegistry,
        round_reg: &RoundRegistry,
    ) -> Standings<StandardScore> {
        match self {
            ScoringSystem::Standard(s) => s.get_standings(player_reg, round_reg),
        }
    }
}
