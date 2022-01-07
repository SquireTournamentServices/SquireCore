use crate::round::Round;

use std::time::Duration;

pub struct RoundRegistry {
    rounds: Vec<Round>,
    length: Duration,
}

impl RoundRegistry {
    pub fn new(len: Duration) -> Self {
        RoundRegistry {
            rounds: Vec::new(),
            length: len,
        }
    }

    pub fn create_match(&mut self) -> () {
        self.rounds
            .push(Round::new(self.rounds.len() as u64, self.length));
    }
}
