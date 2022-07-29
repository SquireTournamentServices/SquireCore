use std::{
    collections::{
        hash_map::{HashMap, Iter},
        HashSet,
    },
    ops::RangeBounds,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use cycle_map::CycleMap;

use crate::{
    error::TournamentError,
    player::PlayerId,
    round::{Round, RoundId, RoundStatus},
};

#[derive(Serialize, Deserialize, Hash, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum RoundIdentifier {
    Id(RoundId),
    Number(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoundRegistry {
    pub num_and_id: CycleMap<RoundId, u64>,
    pub rounds: HashMap<u64, Round>,
    pub opponents: HashMap<PlayerId, HashSet<PlayerId>>,
    pub starting_table: u64,
    pub length: Duration,
}

impl RoundRegistry {
    pub fn new(starting_table: u64, len: Duration) -> Self {
        RoundRegistry {
            num_and_id: CycleMap::new(),
            rounds: HashMap::new(),
            opponents: HashMap::new(),
            starting_table,
            length: len,
        }
    }

    pub(crate) fn get_table_number(&self) -> u64 {
        let range = 0..(self.rounds.len());
        let mut numbers: Vec<u64> = range
            .rev()
            .take_while(|n| !self.rounds.get(&(*n as u64)).unwrap().is_certified())
            .map(|n| self.rounds.get(&(n as u64)).unwrap().table_number)
            .collect();
        if numbers.is_empty() {
            self.starting_table
        } else {
            numbers.push(self.starting_table);
            numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut last = numbers[0];
            for n in numbers {
                if n - last > 1 {
                    return last + 1;
                }
                last = n;
            }
            last + 1
        }
    }

    pub fn kill_round(&mut self, ident: &RoundIdentifier) -> Result<(), TournamentError> {
        let rnd = self
            .get_mut_round(ident)
            .ok_or(TournamentError::RoundLookup)?;
        let players = rnd.get_all_players();
        rnd.kill_round();
        for plyr in &players {
            for p in &players {
                if let Some(opps) = self.opponents.get_mut(plyr) {
                    opps.remove(p);
                }
            }
        }
        Ok(())
    }

    pub fn active_round_count(&self) -> usize {
        self.rounds
            .iter()
            .filter(|(_, r)| !r.is_certified())
            .count()
    }

    pub fn import_round(&mut self, rnd: Round) -> Result<(), TournamentError> {
        if self.num_and_id.contains_left(&rnd.id)
            || self.num_and_id.contains_right(&rnd.match_number)
        {
            Err(TournamentError::RoundLookup)
        } else {
            self.num_and_id.insert(rnd.id.clone(), rnd.match_number);
            self.rounds.insert(rnd.match_number, rnd);
            Ok(())
        }
    }

    pub fn create_round(&mut self) -> RoundIdentifier {
        let match_num = self.rounds.len() as u64;
        let table_number = self.get_table_number();
        let round = Round::new(match_num, table_number, self.length);
        let digest = RoundIdentifier::Id(round.id.clone());
        self.rounds.insert(match_num, round);
        digest
    }

    pub fn add_player_to_round(
        &mut self,
        ident: &RoundIdentifier,
        plyr: PlayerId,
    ) -> Result<(), TournamentError> {
        let round = self
            .get_mut_round(ident)
            .ok_or(TournamentError::RoundLookup)?;
        let players = round.get_all_players();
        round.add_player(plyr.clone());
        if !self.opponents.contains_key(&plyr) {
            self.opponents.insert(plyr.clone(), HashSet::new());
        }
        for p in players {
            self.opponents
                .get_mut(&p)
                .expect("Player should already be in the opponents map.")
                .insert(plyr.clone());
            self.opponents
                .get_mut(&plyr)
                .expect("Player should already be in the opponents map.")
                .insert(p);
        }
        Ok(())
    }

    pub fn get_round_id(&self, ident: &RoundIdentifier) -> Option<RoundId> {
        match ident {
            RoundIdentifier::Id(id) => Some(id.clone()),
            RoundIdentifier::Number(num) => self.num_and_id.get_left(num).cloned(),
        }
    }

    pub(crate) fn get_mut_round(&mut self, ident: &RoundIdentifier) -> Option<&mut Round> {
        match ident {
            RoundIdentifier::Id(id) => {
                let num = self.num_and_id.get_right(id)?;
                self.rounds.get_mut(num)
            }
            RoundIdentifier::Number(num) => self.rounds.get_mut(num),
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
    pub fn get_player_active_round(
        &mut self,
        id: &PlayerId,
    ) -> Result<&mut Round, TournamentError> {
        let mut nums: Vec<u64> = self
            .rounds
            .iter()
            .filter(|(_, r)| r.players.contains(id) && r.is_certified())
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
