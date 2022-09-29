use std::{
    collections::{HashMap, HashSet},
    fmt,
    time::Duration,
};

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::identifiers::RoundId;
use crate::{error::TournamentError, identifiers::PlayerId};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
#[repr(C)]
/// The status of a round has exactly four states. This enum encodes them
pub enum RoundStatus {
    /// The round is still active and nothing has been recorded
    Open,
    /// At least one result has been recorded, but there are players that have yet to certify the
    /// result
    Uncertified,
    /// All results are in and all players have certified the result
    Certified,
    /// The round is no long consider to be part of the tournament, but is not deleted to prevent
    /// naming collisions.
    Dead,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
/// Encodes part of the final result of a round
pub enum RoundResult {
    /// The specified player won N games
    Wins(PlayerId, u8),
    /// There was a drawn game in the round
    Draw(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A "round" might also be known as a "match" in some circles. This contains of at least two
/// players playing at least one games against each other; however, a round can also encode a bye,
/// a free win for exactly one player.
///
/// Each round tracks its start time, expected length, and any extentions. The round clock starts
/// immediately after being created.
///
/// Results are recorded for each player as well as for each drawn game. After that, the results
/// need to be confirmed by all players or by an admin.
///
/// After the results have been confirmed, the round is consider certified and a winner is declared
/// (if possible)
pub struct Round {
    /// The id of the round
    pub id: RoundId,
    /// The match number of the round
    pub match_number: u64,
    /// The table number the round is assigned to (for paper tournaments)
    pub table_number: u64,
    /// The set of players playing against each other
    pub players: HashSet<PlayerId>,
    /// The status of the round
    pub status: RoundStatus,
    /// The winner after certification, if one exists
    pub winner: Option<PlayerId>,
    /// The winner after certification, if one exists
    pub confirmations: HashSet<PlayerId>,
    /// The winner after certification, if one exists
    pub drops: HashSet<PlayerId>,
    /// The winner after certification, if one exists
    pub results: HashMap<PlayerId, u8>,
    /// The winner after certification, if one exists
    pub draws: u8,
    pub(crate) timer: DateTime<Utc>,
    pub(crate) length: Duration,
    pub(crate) extension: Duration,
    pub(crate) is_bye: bool,
}

impl Round {
    /// Creates a new round
    pub fn new(match_num: u64, table_number: u64, len: Duration) -> Self {
        Round {
            id: RoundId::new(Uuid::new_v4()),
            match_number: match_num,
            table_number,
            players: HashSet::with_capacity(4),
            confirmations: HashSet::with_capacity(4),
            results: HashMap::with_capacity(3),
            draws: 0,
            status: RoundStatus::Open,
            drops: HashSet::new(),
            winner: None,
            timer: Utc::now(),
            length: len,
            extension: Duration::from_secs(0),
            is_bye: false,
        }
    }

    /// Calculates the time left in the round, factoring in time extenstions.
    pub fn time_left(&self) -> Duration {
        let length = self.length + self.extension;
        let elapsed = Duration::from_secs((Utc::now() - self.timer).num_seconds() as u64);
        if elapsed < length {
            length - elapsed
        } else {
            Duration::default()
        }
    }

    /// Adds a time extension to the round
    pub fn time_extension(&mut self, dur: Duration) {
        self.extension += dur;
    }

    /// Adds a player to the round
    pub fn add_player(&mut self, player: PlayerId) {
        self.players.insert(player);
    }

    /// Removes a player's need to confirm the result
    pub fn remove_player(&mut self, player: PlayerId) {
        self.drops.insert(player);
    }

    fn verify_result(&self, result: &RoundResult) -> bool {
        match result {
            RoundResult::Wins(p_id, _) => self.players.contains(p_id),
            RoundResult::Draw(_) => true,
        }
    }

    /// Records part of the result of the round.
    pub fn record_result(&mut self, result: RoundResult) -> Result<(), TournamentError> {
        if self.verify_result(&result) {
            self.confirmations.clear();
            match result {
                RoundResult::Wins(p_id, count) => {
                    self.results.insert(p_id, count);
                    let mut max = 0;
                    for (p, num) in self.results.iter() {
                        if *num > max {
                            max = *num;
                            self.winner = Some(*p);
                        } else if *num == max {
                            self.winner = None;
                        }
                    }
                }
                RoundResult::Draw(count) => {
                    self.draws = count;
                }
            }
            Ok(())
        } else {
            Err(TournamentError::PlayerNotInRound)
        }
    }

    /// Confirms the result of the round for a player
    pub fn confirm_round(&mut self, player: PlayerId) -> Result<RoundStatus, TournamentError> {
        use RoundStatus::*;
        if self.status == Dead {
            Err(TournamentError::IncorrectRoundStatus(self.status))
        } else if !self.players.contains(&player) {
            Err(TournamentError::PlayerNotInRound)
        } else if self.drops.contains(&player) {
            Ok(self.status)
        } else {
            self.confirmations.insert(player);
            if self.confirmations.iter().chain(self.drops.iter()).count() == self.players.len() {
                self.status = Certified;
            }
            Ok(self.status)
        }
    }

    /// Make the round irrelavent
    pub fn kill_round(&mut self) {
        self.status = RoundStatus::Dead;
    }

    /// Record the round as a bye. Only works if exactly one player is in the round.
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

    /// Calculates if the round is certified
    pub fn is_certified(&self) -> bool {
        self.status == RoundStatus::Certified
    }

    /// Removes a player's need to confirm the result
    pub fn is_bye(&self) -> bool {
        self.is_bye
    }

    /// Calculates if the round is certified
    pub fn is_active(&self) -> bool {
        match self.status {
            RoundStatus::Open | RoundStatus::Uncertified => true,
            RoundStatus::Certified | RoundStatus::Dead => false,
        }
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
