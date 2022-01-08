use crate::round::Round;

use uuid::Uuid;

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

    pub fn create_round(&mut self) -> Uuid {
        self.rounds
            .push(Round::new(self.rounds.len() as u64, self.length));
        // Safety check: the vector will be non-empty as we just added something to it
        self.rounds.last().unwrap().get_uuid()
    }
}
