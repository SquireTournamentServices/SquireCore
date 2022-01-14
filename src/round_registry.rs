use crate::round::Round;

use uuid::Uuid;

use std::time::Duration;

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

    pub fn create_round(&mut self) -> Uuid {
        self.rounds
            .push(Round::new(self.rounds.len() as u64, self.length));
        // Safety check: the vector will be non-empty as we just added something to it
        self.rounds.last().unwrap().get_uuid()
    }
    
    pub fn get_round(&self, ident: RoundIdentifier) -> Result<&Round, ()> {
        match ident {
            RoundIdentifier::Id(id) => todo!(),
            RoundIdentifier::Number(num) => todo!(),
        }
    }
}
