use std::{
    collections::{hash_map::HashMap, HashSet},
    time::Duration,
};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Seq};

use cycle_map::CycleMap;

use crate::{
    error::TournamentError::{self, NoActiveRound, RoundLookup},
    identifiers::{PlayerId, RoundId},
    pairings::Pairings,
    rounds::Round,
};

use super::RoundContext;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The struct that creates and manages all rounds.
pub struct RoundRegistry {
    /// A lookup table between round ids and match numbers
    pub num_and_id: CycleMap<u64, RoundId>,
    /// All the rounds in a tournament
    #[serde_as(as = "Seq<(_, _)>")]
    pub rounds: HashMap<RoundId, Round>,
    /// A lookup table between players and their opponents. This is duplicate data, but used
    /// heavily by scoring and pairings systems
    #[serde_as(as = "Seq<(_, _)>")]
    pub opponents: HashMap<PlayerId, HashSet<PlayerId>>,
    /// The starting table number for assigning table numbers
    pub starting_table: u64,
    /// The length of new round
    pub length: Duration,
    /// The players' seating scores, for seeded table ordering
    #[serde(default)]
    #[serde_as(as = "Seq<(_, _)>")]
    seat_scores: HashMap<PlayerId, usize>,
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
            seat_scores: HashMap::new(),
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
            .filter_map(|(id, r)| r.contains_player(&p_id).then_some(*id))
            .collect()
    }

    /// Gets a round's id by its match number
    pub fn get_round_id(&self, n: &u64) -> Result<RoundId, TournamentError> {
        self.num_and_id
            .get_right(n)
            .cloned()
            .ok_or(TournamentError::RoundLookup)
    }

    /// Gets a round's id by its match number
    pub fn round_from_table_number(&self, n: u64) -> Result<&Round, TournamentError> {
        self.rounds
            .values()
            .find(|r| r.table_number == n && r.is_active())
            .ok_or(TournamentError::RoundLookup)
    }

    pub(crate) fn get_by_number(&self, n: &u64) -> Result<&Round, TournamentError> {
        self.num_and_id
            .get_right(n)
            .and_then(|id| self.rounds.get(id))
            .ok_or(TournamentError::RoundLookup)
    }

    /// Gets the next table number. Not all pairing systems force all matches to be over before
    /// pairing more players. This ensure new rounds don't the same table number as an active round
    pub(crate) fn get_table_number(&self) -> u64 {
        let mut tracker = self.starting_table;
        self.rounds
            .values()
            .filter_map(|r| r.is_active().then_some(r.table_number))
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
            .unwrap_or(tracker)
    }

    /// Marks a round as dead
    pub fn kill_round(&mut self, ident: &RoundId) -> Result<(), TournamentError> {
        let rnd = self.get_mut_round(ident)?;
        let players = rnd.players.clone();
        rnd.kill_round();
        for (i, plyr) in players.iter().enumerate() {
            self.seat_scores.entry(*plyr).and_modify(|n| *n -= i);
            self.opponents
                .entry(*plyr)
                .and_modify(|opps| opps.retain(|o| !players.contains(o)));
        }
        Ok(())
    }

    /// Calculates the number of rounds that are not confirmed or dead
    pub fn active_round_count(&self) -> usize {
        self.rounds.iter().filter(|(_, r)| r.is_active()).count()
    }

    /// Creates a series of matches from pairings
    pub fn rounds_from_pairings(
        &mut self,
        salt: DateTime<Utc>,
        pairings: Pairings,
        context: RoundContext,
    ) -> Vec<RoundId> {
        let digest = pairings.get_ids(salt);
        pairings
            .paired
            .into_iter()
            .for_each(|p| { self.create_round(salt, p, context.clone()); });
        pairings
            .rejected
            .into_iter()
            .for_each(|p| { self.give_bye(salt, p, context.clone()); });
        digest
    }

    /// Creates a bye and gives it to a player
    pub fn give_bye(
        &mut self,
        salt: DateTime<Utc>,
        plyr: PlayerId,
        context: RoundContext,
    ) -> RoundId {
        let match_num = self.rounds.len() as u64;
        let round = Round::new_bye(salt, plyr, match_num, self.length, context);
        let id = round.id;
        self.num_and_id.insert(match_num, id);
        self.rounds.insert(id, round);
        id
    }

    /// Creates a new round, fills it with players, and returns its id
    pub fn create_round(
        &mut self,
        salt: DateTime<Utc>,
        plyrs: Vec<PlayerId>,
        context: RoundContext,
    ) -> RoundId {
        // Sort players by their prior seating order. Lower seating order is means you last
        let plyrs: Vec<_> = plyrs
            .into_iter()
            .map(|p| (p, *self.seat_scores.entry(p).or_default()))
            .sorted_by(|a, b| a.1.cmp(&b.1).reverse())
            .map(|(p, _)| p)
            .collect();
        for (i, plyr) in plyrs.iter().enumerate() {
            self.seat_scores.entry(*plyr).and_modify(|n| *n += i);
            self.opponents
                .entry(*plyr)
                .or_default()
                .extend(plyrs.iter().filter(|p| *p != plyr));
        }
        let match_num = 1 + self.rounds.len() as u64;
        let table_number = self.get_table_number();
        let round = Round::new(salt, plyrs, match_num, table_number, self.length, context);
        let id = round.id;
        self.num_and_id.insert(match_num, id);
        self.rounds.insert(id, round);
        id
    }

    /// Given a round identifier, returns a round's match number if the round can be found
    pub fn get_round_number(&self, id: &RoundId) -> Result<u64, TournamentError> {
        self.num_and_id.get_left(id).cloned().ok_or(RoundLookup)
    }

    /// Given a round identifier, returns a mutable reference to the round if the round can be found
    pub(crate) fn get_mut_round(&mut self, id: &RoundId) -> Result<&mut Round, TournamentError> {
        self.rounds.get_mut(id).ok_or(RoundLookup)
    }

    /// Given a round identifier, returns a reference to the round if the round can be found
    pub fn get_round(&self, id: &RoundId) -> Result<&Round, TournamentError> {
        self.rounds.get(id).ok_or(RoundLookup)
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
            .ok_or(NoActiveRound)
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

    use chrono::Utc;

    use crate::{
        identifiers::id_from_item,
        rounds::{RoundContext, RoundRegistry, RoundStatus},
    };

    #[test]
    fn table_number_tests() {
        for start in 0..3 {
            let mut reg = RoundRegistry::new(start, Duration::from_secs(10));
            assert_eq!(reg.get_table_number(), start);
            let id_one = reg.create_round(Utc::now(), vec![], RoundContext::Contextless);
            assert_eq!(reg.get_round(&id_one).unwrap().table_number, start);
            assert_eq!(reg.round_from_table_number(start).unwrap().id, id_one);
            assert_eq!(reg.get_table_number(), start + 1);
            let id_two = reg.create_round(Utc::now(), vec![], RoundContext::Contextless);
            assert_eq!(reg.get_round(&id_two).unwrap().table_number, start + 1);
            assert_eq!(reg.round_from_table_number(start + 1).unwrap().id, id_two);
            assert_eq!(reg.get_table_number(), start + 2);
            reg.get_mut_round(&id_one).unwrap().status = RoundStatus::Certified;
            assert_eq!(reg.get_table_number(), start);
            assert!(reg.round_from_table_number(start).is_err());
            let id_three = reg.create_round(Utc::now(), vec![], RoundContext::Contextless);
            assert_eq!(reg.get_round(&id_three).unwrap().table_number, start);
            assert_eq!(reg.round_from_table_number(start).unwrap().id, id_three);
            assert_eq!(reg.get_table_number(), start + 2);
        }
    }

    #[test]
    fn simple_seating_scores_test() {
        let plyrs = vec![
            id_from_item(Utc::now(), Utc::now()),
            id_from_item(Utc::now(), Utc::now()),
        ];
        assert!(plyrs[0] != plyrs[1]);
        let mut reg = RoundRegistry::new(1, Duration::from_secs(10));
        let id = reg.create_round(Utc::now(), plyrs.clone(), RoundContext::Contextless);
        let first_order = reg.get_round(&id).unwrap().players.clone();
        assert_eq!(plyrs, first_order);
        assert_eq!(0, *reg.seat_scores.get(&plyrs[0]).unwrap());
        assert_eq!(1, *reg.seat_scores.get(&plyrs[1]).unwrap());
        let id = reg.create_round(Utc::now(), plyrs.clone(), RoundContext::Contextless);
        let second_order = reg.get_round(&id).unwrap().players.clone();
        assert!(plyrs != second_order);
        assert_eq!(1, *reg.seat_scores.get(&plyrs[0]).unwrap());
        assert_eq!(1, *reg.seat_scores.get(&plyrs[1]).unwrap());
    }
}
