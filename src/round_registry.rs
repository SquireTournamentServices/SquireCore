use crate::round::Round;

use uuid::Uuid;

use std::time::Duration;

#[derive(Debug,Clone)]
pub enum RoundIdentifier {
    Id(Uuid),
    Number(u64),
}

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

    pub fn create_round(&mut self) -> &mut Round {
        self.rounds
            .push(Round::new(self.rounds.len() as u64, self.length));
        // Safety check: the vector will be non-empty as we just added something to it
        &mut self.rounds.last().unwrap()
    }

    pub fn set_round_length(&mut self, length: Duration) -> () {
        self.length = length;
    }

    pub fn verify_identifier(&self, ident: &RoundIdentifier) -> bool {
        match ident {
            RoundIdentifier::Id(id) => self.rounds.iter().any(|m| m.uuid == *id),
            RoundIdentifier::Number(num) => self.rounds.iter().any(|m| m.match_number == *num),
        }
    }

    pub fn get_round(&mut self, ident: RoundIdentifier) -> Result<&mut Round, ()> {
        match ident {
            RoundIdentifier::Id(id) => todo!(),
            RoundIdentifier::Number(num) => todo!(),
        }
    }
}
