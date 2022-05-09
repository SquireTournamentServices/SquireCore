use crate::{error::TournamentError, player::PlayerId};

//use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum RoundStatus {
    Open,
    Uncertified,
    Certified,
    Dead,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RoundResult {
    Wins(PlayerId, u8),
    Draw(),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RoundId(Uuid);

// This struct should be able to handle N many games, unlike the Python equiv.
// TODO: Added back in once alternative for Duration is found
//#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Debug, Clone)]
pub struct Round {
    pub(crate) id: RoundId,
    pub(crate) match_number: u64,
    pub(crate) players: HashSet<PlayerId>,
    confirmations: HashSet<PlayerId>,
    pub(crate) results: Vec<RoundResult>,
    status: RoundStatus,
    pub(crate) winner: Option<PlayerId>,
    timer: Instant,
    length: Duration,
    extension: Duration,
    pub(crate) is_bye: bool,
}

impl Hash for Round {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let _ = &self.id.hash(state);
    }
}

impl PartialEq for Round {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Round {
    pub fn new(match_num: u64, len: Duration) -> Self {
        Round {
            id: RoundId(Uuid::new_v4()),
            match_number: match_num,
            players: HashSet::with_capacity(4),
            confirmations: HashSet::with_capacity(4),
            results: Vec::with_capacity(3),
            status: RoundStatus::Open,
            winner: None,
            timer: Instant::now(),
            length: len,
            extension: Duration::from_secs(0),
            is_bye: false,
        }
    }

    pub fn get_id(&self) -> RoundId {
        self.id
    }

    pub fn add_player(&mut self, player: PlayerId) {
        self.players.insert(player);
    }

    pub fn get_all_players(&self) -> HashSet<PlayerId> {
        self.players.clone()
    }

    fn verify_result(&self, result: &RoundResult) -> bool {
        match result {
            RoundResult::Wins(p_id, _) => self.players.contains(p_id),
            RoundResult::Draw() => true,
        }
    }

    pub fn record_result(&mut self, result: RoundResult) -> Result<(), TournamentError> {
        if self.verify_result(&result) {
            self.results.push(result);
            Ok(())
        } else {
            Err(TournamentError::PlayerNotInRound)
        }
    }

    pub fn confirm_round(&mut self, player: PlayerId) -> Result<RoundStatus, TournamentError> {
        if !self.players.contains(&player) {
            Err(TournamentError::PlayerNotInRound)
        } else {
            self.confirmations.insert(player);
            if self.confirmations.len() == self.players.len() {
                Ok(RoundStatus::Certified)
            } else {
                Ok(RoundStatus::Uncertified)
            }
        }
    }

    pub fn kill_round(&mut self) {
        self.status = RoundStatus::Dead;
    }

    pub fn record_bye(&mut self) -> Result<(), TournamentError> {
        if self.players.len() != 1 {
            Err(TournamentError::InvalidBye)
        } else {
            self.is_bye = true;
            self.winner = Some(self.players.iter().next().unwrap().clone());
            self.status = RoundStatus::Certified;
            Ok(())
        }
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    pub fn is_certified(&self) -> bool {
        self.status == RoundStatus::Certified
    }
}
