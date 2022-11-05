use serde::{Deserialize, Serialize};

use crate::{
    accounts::SquireAccount,
    identifiers::{PlayerId, RoundId},
    rounds::RoundResult,
    settings::TournamentSetting, pairings::Pairings,
};

/// Operations that only tournament admin can perform
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    CreatePairings,
    /// Operation to attempt to pair the next set of rounds
    PairRound(Pairings),
    /// Operation to cut to the top N players (by standings)
    Cut(usize),
    /// Operation to prune excess decks from players
    PruneDecks,
    /// Operation to prune players that aren't fully registered
    PrunePlayers,
}
