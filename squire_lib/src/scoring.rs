use serde::{Deserialize, Serialize};

use crate::identifiers::PlayerId;

/// The trait the defines the interface for a score
pub trait Score
where
    Self: ToString,
{
}

#[derive(Debug, Serialize, Deserialize)]
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
