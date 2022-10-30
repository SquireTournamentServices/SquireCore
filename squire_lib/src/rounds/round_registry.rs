use std::{
    collections::{hash_map::HashMap, HashSet},
    time::Duration,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use cycle_map::CycleMap;

use crate::{
    error::TournamentError::{self, NoActiveRound, RoundLookup},
    identifiers::{PlayerId, RoundId},
    rounds::Round,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The struct that creates and manages all rounds.
pub struct RoundRegistry {
    /// A lookup table between round ids and match numbers
    pub num_and_id: CycleMap<u64, RoundId>,
    /// All the rounds in a tournament
    pub rounds: HashMap<RoundId, Round>,
    /// A lookup table between players and their opponents. This is duplicate data, but used
    /// heavily by scoring and pairings systems
    pub opponents: HashMap<PlayerId, HashSet<PlayerId>>,
    /// The starting table number for assigning table numbers
    pub starting_table: u64,
    /// The length of new round
    pub length: Duration,
}

impl RoundRegistry {
    /// Creates a new round registry
    pub fn new(starting_table: u64, len: Duration) -> Self {
        RoundRegistry {
            num_and_id: CycleMap::new(),
            rounds: HashMap::new(),
            opponents: HashMap::new(),
            starting_table,
            length: len,
        }
    }

    /// Determines if the given id corresponds to a round in this registry
    pub fn validate_id(&self, r_id: &RoundId) -> bool {
        self.rounds.contains_key(r_id)
    }

    /// Returns a list of copied round ids for a player, this is used in FFI mostly.
    pub fn get_round_ids_for_player(&self, p_id: PlayerId) -> Vec<RoundId> {
        self.rounds
            .iter()
            .filter_map(|(id, r)| r.contains_player(&p_id).then(|| *id))
            .collect()
    }

    /// Gets a round's id by its match number
    pub fn get_round_id(&self, n: &u64) -> Result<RoundId, TournamentError> {
        self.num_and_id
            .get_right(n)
            .cloned()
            .ok_or_else(|| TournamentError::RoundLookup)
    }

    pub(crate) fn get_by_number(&self, n: &u64) -> Result<&Round, TournamentError> {
        self.num_and_id
            .get_right(n)
            .map(|id| self.rounds.get(id))
            .flatten()
            .ok_or_else(|| TournamentError::RoundLookup)
    }

    /// Gets the next table number. Not all pairing systems force all matches to be over before
    /// pairing more players. This ensure new rounds don't the same table number as an active round
    pub(crate) fn get_table_number(&self) -> u64 {
        let mut tracker = self.starting_table;
        self.rounds
            .values()
            .filter_map(|r| r.is_active().then(|| r.table_number))
            .sorted()
            .zip(self.starting_table..(self.rounds.len() as u64 + self.starting_table))
            .find_map(|(active, new)| {
                if active == new {
                    tracker += 1;
                    None
                } else {
                    Some(new)
                }
            })
            .unwrap_or_else(|| tracker)
    }

    /// Marks a round as dead
    pub fn kill_round(&mut self, ident: &RoundId) -> Result<(), TournamentError> {
        let rnd = self.get_mut_round(ident)?;
        let players = rnd.players.clone();
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

    /// Calculates the number of rounds that are not confirmed or dead
    pub fn active_round_count(&self) -> usize {
        self.rounds.iter().filter(|(_, r)| r.is_active()).count()
    }
    
    /// Updates a round's id
    pub(crate) fn update_id(&mut self, old_id: RoundId, new_id: RoundId) -> bool {
        match self.rounds.remove(&old_id) {
            Some(mut rnd) => {
                rnd.id = new_id;
                self.rounds.insert(new_id, rnd);
                true
            }
            None => false,
        }
    }

    /// Creates a new round with no players and returns its id
    pub fn create_round(&mut self) -> RoundId {
        let match_num = self.rounds.len() as u64;
        let table_number = self.get_table_number();
        let round = Round::new(match_num, table_number, self.length);
        let digest = round.id.into();
        self.num_and_id.insert(match_num, round.id);
        self.rounds.insert(round.id, round);
        digest
    }

    /// Adds a player to the specified round
    pub fn add_player_to_round(
        &mut self,
        ident: &RoundId,
        plyr: PlayerId,
    ) -> Result<(), TournamentError> {
        let ident = (*ident).into();
        let round = self.get_mut_round(&ident)?;
        let players = round.players.clone();
        round.add_player(plyr);
        self.opponents.entry(plyr).or_insert_with(HashSet::new);
        for p in players {
            self.opponents
                .get_mut(&p)
                .expect("Player should already be in the opponents map.")
                .insert(plyr);
            self.opponents
                .get_mut(&plyr)
                .expect("Player should already be in the opponents map.")
                .insert(p);
        }
        Ok(())
    }

    /// Given a round identifier, returns a round's match number if the round can be found
    pub fn get_round_number(&self, id: &RoundId) -> Result<u64, TournamentError> {
        self.num_and_id.get_left(id).cloned().ok_or(RoundLookup)
    }

    /// Given a round identifier, returns a mutable reference to the round if the round can be found
    pub(crate) fn get_mut_round(&mut self, id: &RoundId) -> Result<&mut Round, TournamentError> {
        self.rounds.get_mut(&id).ok_or(RoundLookup)
    }

    /// Given a round identifier, returns a reference to the round if the round can be found
    pub fn get_round(&self, id: &RoundId) -> Result<&Round, TournamentError> {
        self.rounds.get(&id).ok_or(RoundLookup)
    }

    /// This is a messy function... but the idea was ported directly from the Python version
    /// It is theoretically possible for a player to end up in more than one active match (unlikely,
    /// but we must prepare for the worst). Should this ever happen, we return the "oldest" active
    /// match of theirs. However, this is FAR from ideal as every match certification requires a
    /// pass through all matches... gross.
    ///
    /// Potentail clean up: We can likely avoid this be maintaining that a player can be in at most
    /// one match at a time. We can then use a GroupMap to look up match ids via player ids.
    pub fn get_player_active_round(
        &mut self,
        id: &PlayerId,
    ) -> Result<&mut Round, TournamentError> {
        self.rounds
            .values_mut()
            .filter(|r| r.players.contains(id) && r.is_active())
            .sorted_by(|a, b| a.match_number.cmp(&b.match_number))
            .next()
            .ok_or_else(|| NoActiveRound)
    }

    /// Gets a Vec of `&mut Round`
    pub fn get_player_active_rounds(&mut self, id: &PlayerId) -> Vec<&mut Round> {
        self.rounds
            .iter_mut()
            .map(|(_, r)| r)
            .filter(|r| r.players.contains(id) && r.is_active())
            .collect()
    }

    /// Sets the length for new rounds
    pub fn set_round_length(&mut self, length: Duration) {
        self.length = length;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::rounds::RoundStatus;

    use super::RoundRegistry;

    #[test]
    fn table_number_tests() {
        for start in 0..3 {
            let mut reg = RoundRegistry::new(start, Duration::from_secs(10));
            assert_eq!(reg.get_table_number(), start);
            let id_one = reg.create_round();
            assert_eq!(reg.get_round(&id_one).unwrap().table_number, start);
            assert_eq!(reg.get_table_number(), start + 1);
            let id_two = reg.create_round();
            assert_eq!(reg.get_round(&id_two).unwrap().table_number, start + 1);
            assert_eq!(reg.get_table_number(), start + 2);
            reg.get_mut_round(&id_one).unwrap().status = RoundStatus::Certified;
            assert_eq!(reg.get_table_number(), start);
            let id_three = reg.create_round();
            assert_eq!(reg.get_round(&id_three).unwrap().table_number, start);
            assert_eq!(reg.get_table_number(), start + 2);
        }
    }
}
