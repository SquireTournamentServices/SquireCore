use crate::error::TournamentError;
use crate::round::Round;

use uuid::Uuid;

use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::ops::RangeBounds;
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

    pub fn iter(&self) -> Iter<u64, Round> {
        self.rounds.iter()
    }

    pub fn create_round(&mut self) -> &mut Round {
        let match_num = self.rounds.len() as u64;
        self.rounds
            .insert(match_num, Round::new(match_num, self.length));
        // Safety check: We just inserted a round with the key match_num. It's there
        self.rounds.get_mut(&match_num).unwrap()
    }

    pub fn get_mut_round(&mut self, ident: RoundIdentifier) -> Result<&mut Round, TournamentError> {
        let num = self.get_round_number(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.rounds.get_mut(&num).unwrap())
    }

    pub fn get_round(&self, ident: RoundIdentifier) -> Result<&Round, TournamentError> {
        let num = self.get_round_number(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.rounds.get(&num).unwrap())
    }

    // This is a messy function... but the idea was ported directly from the Python version
    // It is theoretically possible for a player to end up in more than one active match (unlikely,
    // but we must prepare for the worst). Should this ever happen, we return the "oldest" active
    // match of theirs. However, this is FAR from ideal as every match certification requires a
    // pass through all matches... gross.
    pub fn get_player_active_round(&mut self, id: Uuid) -> Result<&mut Round, TournamentError> {
        let mut nums: Vec<u64> = self
            .rounds
            .iter()
            .filter(|(_, r)| r.players.contains(&id) && r.is_certified())
            .map(|(_, r)| r.match_number)
            .collect();
        nums.sort_unstable();
        if nums.is_empty() {
            Err(TournamentError::NoActiveRound)
        } else {
            Ok(self.rounds.get_mut(&nums[0]).unwrap())
        }
    }

    pub fn set_round_length(&mut self, length: Duration) {
        self.length = length;
    }

    pub fn get_round_number(&self, ident: RoundIdentifier) -> Result<u64, TournamentError> {
        match ident {
            RoundIdentifier::Id(id) => {
                if self.verify_identifier(&RoundIdentifier::Id(id)) {
                    let nums: Vec<u64> = self
                        .rounds
                        .iter()
                        .filter(|(_, r)| r.uuid == id)
                        .map(|(i, _)| *i)
                        .collect();
                    // Safety check: We verified identifiers above, so there is a round with the
                    // given id.
                    Ok(nums[0])
                } else {
                    Err(TournamentError::RoundLookup)
                }
            }
            RoundIdentifier::Number(num) => {
                if self.rounds.contains_key(&num) {
                    Ok(num)
                } else {
                    Err(TournamentError::RoundLookup)
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
}
