use serde::{Deserialize, Serialize};

use crate::identifiers::PlayerId;

/// The trait the defines the interface for a score
pub trait Score
where
    Self: ToString,
{
    /// Returns the primary value of the score
    fn primary_score(&self) -> f64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
