use std::{
    collections::HashSet,
    fmt,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::identifiers::RoundId;
use crate::{error::TournamentError, identifiers::PlayerId};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
#[repr(C)]
pub enum RoundStatus {
    Open,
    Uncertified,
    Certified,
    Dead,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub enum RoundResult {
    Wins(PlayerId, u8),
    Draw(),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Round {
    pub id: RoundId,
    pub match_number: u64,
    pub table_number: u64,
    pub players: HashSet<PlayerId>,
    pub(crate) confirmations: HashSet<PlayerId>,
    pub(crate) results: Vec<RoundResult>,
    pub status: RoundStatus,
    pub winner: Option<PlayerId>,
    pub(crate) timer: SystemTime,
    pub(crate) length: Duration,
    pub(crate) extension: Duration,
    pub(crate) is_bye: bool,
}

impl Round {
    pub fn new(match_num: u64, table_number: u64, len: Duration) -> Self {
        Round {
            id: RoundId::new(Uuid::new_v4()),
            match_number: match_num,
            table_number,
            players: HashSet::with_capacity(4),
            confirmations: HashSet::with_capacity(4),
            results: Vec::with_capacity(3),
            status: RoundStatus::Open,
            winner: None,
            timer: SystemTime::now(),
            length: len,
            extension: Duration::from_secs(0),
            is_bye: false,
        }
    }

    // TODO: Find a better way to sync clocks if SystemTime::elapsed errors
    pub fn time_left(&self) -> Duration {
        let length = self.length + self.extension;
        let elapsed = match self.timer.elapsed() {
            Ok(e) => e,
            Err(_) => {
                return Duration::from_secs(0);
            }
        };
        if elapsed > length {
            Duration::from_secs(0)
        } else {
            length - elapsed
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
            self.winner = Some(*self.players.iter().next().unwrap());
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

impl fmt::Display for RoundStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Open => "Open",
                Self::Uncertified => "Uncertified",
                Self::Certified => "Certified",
                Self::Dead => "Dead",
            }
        )
    }
}
