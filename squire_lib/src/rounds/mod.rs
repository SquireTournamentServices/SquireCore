use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt,
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Seq};

pub use crate::identifiers::RoundId;
use crate::{
    error::TournamentError,
    identifiers::{id_from_item, id_from_list, PlayerId, RoundIdentifier},
    pairings::swiss_pairings::SwissContext,
};

mod round_registry;
pub use round_registry::RoundRegistry;

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
#[repr(C)]
/// The status of a round has exactly four states. This enum encodes them
pub enum RoundStatus {
    /// The round is still active and nothing has been recorded
    #[default]
    Open,
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
    Wins(PlayerId, u32),
    /// There was a drawn game in the round
    Draw(u32),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash, PartialEq, Eq)]
/// The context in which the round was created
pub enum RoundContext {
    /// No additional context available
    #[default]
    Contextless,
    /// The context from the swiss pairings
    Swiss(SwissContext),
    /// The context from multiple sources
    Multiple(Vec<RoundContext>),
}

#[serde_as]
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
    pub players: Vec<PlayerId>,
    /// The status of the round
    pub status: RoundStatus,
    /// The winner after certification, if one exists
    pub winner: Option<PlayerId>,
    /// The winner after certification, if one exists
    pub confirmations: HashSet<PlayerId>,
    /// The winner after certification, if one exists
    pub drops: HashSet<PlayerId>,
    /// The winner after certification, if one exists
    #[serde_as(as = "Seq<(_, _)>")]
    pub results: HashMap<PlayerId, u32>,
    /// The winner after certification, if one exists
    pub draws: u32,
    /// The round context that the round was created in
    #[serde(default)]
    pub context: RoundContext,
    /// The start time of the round
    pub timer: DateTime<Utc>,
    /// The length of the round
    pub length: Duration,
    /// All recorded time extensions for the round
    pub extension: Duration,
    /// Whether or not this round is a bye
    pub is_bye: bool,
}

impl Round {
    /// Creates a new round
    pub fn new(
        salt: DateTime<Utc>,
        players: Vec<PlayerId>,
        match_num: u64,
        table_number: u64,
        len: Duration,
        context: RoundContext,
    ) -> Self {
        let id = Self::create_id(salt, &players);
        let confirmations = HashSet::with_capacity(players.len());
        let results = HashMap::with_capacity(players.len());
        Round {
            id,
            match_number: match_num,
            table_number,
            players,
            confirmations,
            results,
            context,
            draws: 0,
            timer: salt,
            length: len,
            status: RoundStatus::Open,
            drops: HashSet::new(),
            winner: None,
            extension: Duration::from_secs(0),
            is_bye: false,
        }
    }

    pub(crate) fn create_id(salt: DateTime<Utc>, players: &[PlayerId]) -> RoundId {
        id_from_list(salt, players.iter())
    }

    /// Creates a new bye round
    pub fn new_bye(
        salt: DateTime<Utc>,
        plyr: PlayerId,
        match_num: u64,
        len: Duration,
        context: RoundContext,
    ) -> Self {
        Round {
            id: id_from_item(salt, plyr),
            match_number: match_num,
            table_number: 0,
            players: vec![plyr],
            confirmations: HashSet::new(),
            results: HashMap::new(),
            draws: 0,
            status: RoundStatus::Certified,
            drops: HashSet::new(),
            winner: Some(plyr),
            timer: salt,
            length: len,
            extension: Duration::from_secs(0),
            is_bye: true,
            context,
        }
    }

    /// Calculates if an identifier matches data in this round
    pub fn match_ident(&self, ident: RoundIdentifier) -> bool {
        match ident {
            RoundIdentifier::Id(id) => self.id == id,
            RoundIdentifier::Number(num) => self.match_number == num,
            RoundIdentifier::Table(num) => self.table_number == num,
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

    /// Removes a player's need to confirm the result
    pub fn drop_player(&mut self, plyr: &PlayerId) {
        self.drops.retain(|p| p != plyr);
    }

    /// Calculates if there is a result recorded for the match
    pub fn has_result(&self) -> bool {
        self.draws != 0 || self.results.values().sum::<u32>() != 0
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
            if self.is_active() {
                self.confirmations.clear();
            }
            match result {
                RoundResult::Wins(p_id, count) => {
                    self.results.insert(p_id, count);
                    let mut max = 0;
                    for (p, num) in self.results.iter() {
                        match max.cmp(num) {
                            Ordering::Less => {
                                max = *num;
                                self.winner = Some(*p);
                            }
                            Ordering::Equal => {
                                self.winner = None;
                            }
                            Ordering::Greater => {}
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
        } else if !self.has_result() {
            Err(TournamentError::NoMatchResult)
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
            RoundStatus::Open => true,
            RoundStatus::Certified | RoundStatus::Dead => false,
        }
    }

    /// Calcualtes if a player is in the round (active or dropped).
    pub fn contains_player(&self, p_id: &PlayerId) -> bool {
        self.players.contains(p_id) || self.drops.contains(p_id)
    }
}

impl fmt::Display for RoundStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Open => "Open",
                Self::Certified => "Certified",
                Self::Dead => "Dead",
            }
        )
    }
}

impl RoundContext {
    /// Combines two round contexts
    pub fn combine(self, other: Self) -> Self {
        use RoundContext::*;
        match self {
            Contextless => other,
            Swiss(ctx) => match other {
                Contextless | Swiss(_) => Swiss(ctx),
                Multiple(mut context) => {
                    context.push(Swiss(ctx));
                    Multiple(context)
                }
            },
            Multiple(mut ctx) => match other {
                Contextless => Multiple(ctx),
                Swiss(context) => {
                    ctx.push(Swiss(context));
                    Multiple(ctx)
                }
                Multiple(context) => {
                    ctx.extend(context);
                    Multiple(ctx)
                }
            },
        }
    }
}
