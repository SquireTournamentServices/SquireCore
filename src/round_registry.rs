use crate::{
    error::TournamentError,
    player::PlayerId,
    round::{Round, RoundId},
};

use cycle_map::CycleMap;

use serde::{Deserialize, Serialize};

use std::{
    collections::hash_map::{HashMap, Iter},
    ops::RangeBounds,
    time::Duration,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum RoundIdentifier {
    Id(RoundId),
    Number(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoundRegistry {
    pub(crate) num_and_id: CycleMap<RoundId, u64>,
    pub(crate) rounds: HashMap<u64, Round>,
    length: Duration,
}

impl RoundRegistry {
    pub fn new(len: Duration) -> Self {
        RoundRegistry {
            num_and_id: CycleMap::new(),
            rounds: HashMap::new(),
            length: len,
        }
    }

    pub fn active_round_count(&self) -> usize {
        self.rounds
            .iter()
            .filter(|(_, m)| !m.is_certified())
            .count()
    }

    pub fn create_round(&mut self) -> &mut Round {
        let match_num = self.rounds.len() as u64;
        self.rounds
            .insert(match_num, Round::new(match_num, self.length));
        // Safety check: We just inserted a round with the key match_num. It's there
        self.rounds.get_mut(&match_num).unwrap()
    }

    pub fn get_mut_round(&mut self, ident: RoundIdentifier) -> Option<&mut Round> {
        match ident {
            RoundIdentifier::Id(id) => {
                let num = self.num_and_id.get_right(&id)?;
                self.rounds.get_mut(num)
            }
            RoundIdentifier::Number(num) => self.rounds.get_mut(&num),
        }
    }

    pub fn get_round(&self, ident: &RoundIdentifier) -> Option<&Round> {
        match ident {
            RoundIdentifier::Id(id) => {
                let num = self.num_and_id.get_right(id)?;
                self.rounds.get(num)
            }
            RoundIdentifier::Number(num) => self.rounds.get(num),
        }
    }

    // This is a messy function... but the idea was ported directly from the Python version
    // It is theoretically possible for a player to end up in more than one active match (unlikely,
    // but we must prepare for the worst). Should this ever happen, we return the "oldest" active
    // match of theirs. However, this is FAR from ideal as every match certification requires a
    // pass through all matches... gross.
    //
    // Potentail clean up: We can likely avoid this be maintaining that a player can be in at most
    // one match at a time. We can then use a GroupMap to look up match ids via player ids.
    pub fn get_player_active_round(&mut self, id: PlayerId) -> Result<&mut Round, TournamentError> {
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
}
