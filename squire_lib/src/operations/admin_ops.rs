use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    accounts::SquireAccount,
    identifiers::{PlayerId, RoundId},
    operations::OpUpdate,
    pairings::Pairings,
    rounds::{Round, RoundResult},
    settings::TournamentSetting,
};

/// Operations that only tournament admin can perform
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum AdminOp {
    /// Operation to check the registration status of the tournament
    UpdateReg(bool),
    /// Operation to start a tournament
    Start,
    /// Operation to freeze a tournament
    Freeze,
    /// Operation to thaw a tournament
    Thaw,
    /// Operation to end a tournament
    End,
    /// Operation to cancel a tournament
    Cancel,
    /// Operation to overwrite the result of a round via an admin (used after a confirmation)
    AdminOverwriteResult(RoundId, RoundResult),
    /// Operation for adding a new judge to the tournament
    RegisterJudge(SquireAccount),
    /// Operation for adding a new tournament admin
    RegisterAdmin(SquireAccount),
    /// Operation to drop a player via an admin
    AdminDropPlayer(PlayerId),
    /// Operation to kill a round
    RemoveRound(RoundId),
    /// Operation to update a single tournament setting
    UpdateTournSetting(TournamentSetting),
    /// Operation to give a player a bye
    GiveBye(PlayerId),
    /// Operation to manually create a round
    CreateRound(Vec<PlayerId>),
    /// Operation to attempt to pair the next set of rounds
    PairRound(Pairings),
    /// Operation to cut to the top N players (by standings)
    Cut(usize),
    /// Operation to prune excess decks from players
    PruneDecks,
    /// Operation to prune players that aren't fully registered
    PrunePlayers,
    /// Operation to confirm the results of all active rounds
    ConfirmAllRounds,
}

impl AdminOp {
    pub(crate) fn get_update(&self, salt: DateTime<Utc>) -> OpUpdate {
        match self {
            AdminOp::GiveBye(plyr) => OpUpdate::RoundId(vec![Round::create_id(salt, &[*plyr])]),
            AdminOp::CreateRound(plyrs) => OpUpdate::RoundId(vec![Round::create_id(salt, plyrs)]),
            AdminOp::PairRound(pairings) => OpUpdate::RoundId(pairings.get_ids(salt)),
            _ => OpUpdate::None,
        }
    }

    pub(crate) fn swap_player_ids(&mut self, old: PlayerId, new: PlayerId) {
        match self {
            AdminOp::AdminDropPlayer(p_id) | AdminOp::GiveBye(p_id) if *p_id == old => {
                *p_id = new;
            }
            AdminOp::CreateRound(plyrs) => {
                plyrs.iter_mut().filter(|p| **p == old).for_each(|p| {
                    *p = new;
                });
            }
            AdminOp::PairRound(pairings) => {
                pairings.swap_player_ids(old, new);
            }
            _ => {}
        }
    }

    pub(crate) fn swap_round_ids(&mut self, old: RoundId, new: RoundId) {
        match self {
            AdminOp::AdminOverwriteResult(r_id, _) | AdminOp::RemoveRound(r_id) if *r_id == old => {
                *r_id = new;
            }
            _ => {}
        }
    }
}
