use std::{
    collections::{hash_map::HashMap, HashSet},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use cycle_map::CycleMap;

use crate::{
    error::TournamentError,
    identifiers::{PlayerId, RoundId},
    round::Round,
};

use TournamentError::{NoActiveRound, RoundLookup};

pub use crate::identifiers::RoundIdentifier;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The struct that creates and manages all rounds.
pub struct RoundRegistry {
    /// A lookup table between round ids and match numbers
    // TODO: We don't need this. A GroupMap between RoundIdentifiers and Rounds would suffice for
    // the rounds field
    pub num_and_id: CycleMap<RoundId, u64>,
    /// All the rounds in a tournament
    pub rounds: HashMap<u64, Round>,
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

    /// Returns a list of copied round ids, this is used in FFI mostly.
    pub fn get_round_ids(&self) -> Vec<RoundId> {
        let mut ret: Vec<RoundId> = Vec::new();
        self.rounds.iter().for_each(|(_, rnd)| ret.push(rnd.id));
        ret
    }

    /// Returns a list of copied round ids for a player, this is used in FFI mostly.
    pub fn get_round_ids_for_player(&self, pid: PlayerId) -> Vec<RoundId> {
        let mut ret: Vec<RoundId> = Vec::new();
        self.rounds.iter().for_each(|(_, rnd)| {
            if rnd.players.contains(&pid) {
                ret.push(rnd.id)
            }
        });
        ret
    }

    /// Gets the next table number. Not all pairing systems force all matches to be over before
    /// pairing more players. This ensure new rounds don't the same table number as an active round
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

    /// Marks a round as dead
    pub fn kill_round(&mut self, ident: &RoundIdentifier) -> Result<(), TournamentError> {
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

    /* TODO: Is this needed?
    pub fn import_round(&mut self, rnd: Round) -> Result<(), TournamentError> {
        if self.num_and_id.contains_left(&rnd.id)
            || self.num_and_id.contains_right(&rnd.match_number)
        {
            Err(TournamentError::RoundLookup)
        } else {
            self.num_and_id.insert(rnd.id, rnd.match_number);
            self.rounds.insert(rnd.match_number, rnd);
            Ok(())
        }
    }
    */

    /// Creates a new round with no players and returns its id
    pub fn create_round(&mut self) -> RoundIdentifier {
        let match_num = self.rounds.len() as u64;
        let table_number = self.get_table_number();
        let round = Round::new(match_num, table_number, self.length);
        let digest = round.id.into();
        self.num_and_id.insert(round.id, match_num);
        self.rounds.insert(match_num, round);
        digest
    }

    /// Adds a player to the specified round
    pub fn add_player_to_round(
        &mut self,
        ident: &RoundIdentifier,
        plyr: PlayerId,
    ) -> Result<(), TournamentError> {
        let round = self.get_mut_round(ident)?;
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

    /// Given a round identifier, returns a round id if the round can be found
    pub fn get_round_id(&self, ident: &RoundIdentifier) -> Result<RoundId, TournamentError> {
        match ident {
            RoundIdentifier::Id(id) => Ok(*id),
            RoundIdentifier::Number(num) => {
                self.num_and_id.get_left(num).cloned().ok_or(RoundLookup)
            }
        }
    }

    /// Given a round identifier, returns a round's match number if the round can be found
    pub fn get_round_number(&self, ident: &RoundIdentifier) -> Result<u64, TournamentError> {
        match ident {
            RoundIdentifier::Number(num) => Ok(*num),
            RoundIdentifier::Id(id) => self.num_and_id.get_right(id).cloned().ok_or(RoundLookup),
        }
    }

    /// Given a round identifier, returns a mutable reference to the round if the round can be found
    pub(crate) fn get_mut_round(
        &mut self,
        ident: &RoundIdentifier,
    ) -> Result<&mut Round, TournamentError> {
        let num = self.get_round_number(ident)?;
        self.rounds.get_mut(&num).ok_or(RoundLookup)
    }

    /// Given a round identifier, returns a reference to the round if the round can be found
    pub fn get_round(&self, ident: &RoundIdentifier) -> Result<&Round, TournamentError> {
        let num = self.get_round_number(ident)?;
        self.rounds.get(&num).ok_or(RoundLookup)
    }

    // TODO: Rework
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
        let mut nums: Vec<u64> = self
            .rounds
            .iter()
            .filter(|(_, r)| r.players.contains(id) && r.is_active())
            .map(|(_, r)| r.match_number)
            .collect();
        nums.sort_unstable();
        if nums.is_empty() {
            Err(NoActiveRound)
        } else {
            Ok(self.rounds.get_mut(&nums[0]).unwrap())
        }
    }

    // TODO: Rework
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
