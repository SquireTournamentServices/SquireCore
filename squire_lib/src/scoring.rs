use serde::{Deserialize, Serialize};

use crate::identifiers::PlayerId;

pub trait Score
where
    Self: ToString,
{
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Standings<S> {
    pub scores: Vec<(PlayerId, S)>,
}

impl<S> Standings<S>
where
    S: Score,
{
    pub fn new(scores: Vec<(PlayerId, S)>) -> Self {
        Standings { scores }
    }
}
