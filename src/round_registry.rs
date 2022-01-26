use crate::round::Round;

use uuid::Uuid;

use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum RoundIdentifier {
    Id(Uuid),
    Number(u64),
}

pub struct RoundRegistry {
    rounds: HashMap<u64, Round>,
    length: Duration,
}

impl RoundRegistry {
    pub fn new(len: Duration) -> Self {
        RoundRegistry {
            rounds: HashMap::new(),
            length: len,
        }
    }

    pub fn create_round(&mut self) -> &mut Round {
        let match_num = self.rounds.len() as u64;
        self.rounds
            .insert(match_num, Round::new(match_num, self.length));
        // Safety check: We just inserted a round with the key match_num. It's there
        self.rounds.get_mut(&match_num).unwrap()
    }

    pub fn get_mut_round(&mut self, ident: RoundIdentifier) -> Result<&mut Round, ()> {
        let num = self.get_round_number(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.rounds.get_mut(&num).unwrap())
    }

    pub fn get_round(&self, ident: RoundIdentifier) -> Result<&Round, ()> {
        let num = self.get_round_number(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.rounds.get(&num).unwrap())
    }

    pub fn set_round_length(&mut self, length: Duration) -> () {
        self.length = length;
    }

    pub fn get_round_number(&self, ident: RoundIdentifier) -> Result<u64, ()> {
        match ident {
            RoundIdentifier::Id(id) => {
                if self.verify_identifier(&RoundIdentifier::Id(id)) {
                    let nums: Vec<u64> = self
                        .rounds
                        .iter()
                        .filter(|(_, r)| r.uuid == id)
                        .map(|(i, _)| i.clone())
                        .collect();
                    // Safety check: We verified identifiers above, so there is a round with the
                    // given id.
                    Ok(nums[0])
                } else {
                    Err(())
                }
            }
            RoundIdentifier::Number(num) => {
                if self.rounds.contains_key(&num) {
                    Ok(num)
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn verify_identifier(&self, ident: &RoundIdentifier) -> bool {
        match ident {
            RoundIdentifier::Id(id) => self.rounds.iter().any(|(_, r)| r.uuid == *id),
            RoundIdentifier::Number(num) => self.rounds.contains_key(num),
        }
    }

    pub fn kill_round(&mut self, ident: RoundIdentifier) -> Result<(), ()> {
        todo!()
    }
}
