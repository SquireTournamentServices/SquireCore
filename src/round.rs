use std::{collections::HashSet, fmt, hash::{Hash, Hasher}, time::{Duration, Instant, SystemTime}};

use uuid::Uuid;
use serde::{Deserialize, Serialize, 
    ser::{Serializer, SerializeStruct},
};

use crate::{error::TournamentError, player::PlayerId};

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

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct RoundId(Uuid);

#[derive(Serialize, Deserialize, Debug, Clone)]
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
            id: RoundId(Uuid::new_v4()),
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
        self.id.clone()
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
/*
pub struct Round {
}

impl Serialize for Round {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Round", 12)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("match_number", &self.match_number)?;
        state.serialize_field("table_number", &self.table_number)?;
        state.serialize_field("players", &self.players)?;
        state.serialize_field("confirmations", &self.confirmations)?;
        state.serialize_field("results", &self.results)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("winner", &self.winner)?;
        state.serialize_field("timer", &self.timer)?;
        state.serialize_field("length", &self.length)?;
        state.serialize_field("extension", &self.extension)?;
        state.serialize_field("is_bye", &self.is_bye)?;
        state.end()
    }
}
*/

impl fmt::Display for RoundStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Open => "Open",
            Self::Uncertified => "Uncertified",
            Self::Certified => "Certified",
            Self::Dead => "Dead",
        })
    }
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
