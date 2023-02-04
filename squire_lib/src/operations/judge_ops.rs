use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    accounts::SquireAccount,
    admin::TournOfficialId,
    identifiers::{PlayerId, RoundId},
    operations::OpGroup,
    players::{Deck, Player},
    rounds::RoundResult,
};

use super::OpUpdate;

/// Operations that judges and tournament admin can perform
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum JudgeOp {
    /// Operation for adding a guest player to a tournament (i.e. someone without an account)
    RegisterGuest(String),
    /// Operation for re-registering a guest
    ReRegisterGuest(String),
    /// Operation to register a player via an admin
    AdminRegisterPlayer(SquireAccount),
    /// Operation to record the result of a round via an admin
    AdminRecordResult(RoundId, RoundResult),
    /// Operation to confirm the result of a round via an admin
    AdminConfirmResult(RoundId, PlayerId),
    /// Operation to add a deck for a player via an admin
    AdminAddDeck(PlayerId, String, Deck),
    /// Operation to remove a deck for a player via an admin
    AdminRemoveDeck(PlayerId, String),
    /// Operation to mark a player as ready for their next round via an admin
    AdminReadyPlayer(PlayerId),
    /// Operation to mark a player as unready for their next round via an admin
    AdminUnReadyPlayer(PlayerId),
    /// Operation to give a round a time extension
    TimeExtension(RoundId, Duration),
    /// Confirms the round result for all players
    ConfirmRound(RoundId),
}

impl JudgeOp {
    pub(crate) fn get_update(&self, salt: DateTime<Utc>) -> OpUpdate {
        match self {
            JudgeOp::RegisterGuest(name) => OpUpdate::PlayerId(Player::create_guest_id(salt, name)),
            _ => OpUpdate::None,
        }
    }

    pub(crate) fn swap_player_ids(&mut self, old: PlayerId, new: PlayerId) {
        match self {
            JudgeOp::AdminConfirmResult(_, p_id)
            | JudgeOp::AdminAddDeck(p_id, _, _)
            | JudgeOp::AdminRemoveDeck(p_id, _)
            | JudgeOp::AdminReadyPlayer(p_id)
            | JudgeOp::AdminUnReadyPlayer(p_id)
                if *p_id == old =>
            {
                *p_id = new;
            }
            _ => {}
        }
    }

    pub(crate) fn swap_round_ids(&mut self, old: RoundId, new: RoundId) {
        match self {
            JudgeOp::AdminRecordResult(r_id, _)
            | JudgeOp::AdminConfirmResult(r_id, _)
            | JudgeOp::TimeExtension(r_id, _)
            | JudgeOp::ConfirmRound(r_id)
                if *r_id == old =>
            {
                *r_id = new;
            }
            _ => {}
        }
    }

    pub(crate) fn affects(&self) -> OpGroup {
        match self {
            JudgeOp::RegisterGuest(name) => {
                todo!()
            }
            JudgeOp::ReRegisterGuest(name) => {
                todo!()
            }
            JudgeOp::AdminRegisterPlayer(account) => {
                todo!()
            }
            JudgeOp::AdminRecordResult(r_id, result) => {
                todo!()
            }
            JudgeOp::AdminConfirmResult(r_id, p_id) => {
                todo!()
            }
            JudgeOp::AdminAddDeck(p_id, name, deck) => {
                todo!()
            }
            JudgeOp::AdminRemoveDeck(p_id, name) => {
                todo!()
            }
            JudgeOp::AdminReadyPlayer(p_id) => {
                todo!()
            }
            JudgeOp::AdminUnReadyPlayer(p_id) => {
                todo!()
            }
            JudgeOp::TimeExtension(r_id, dur) => {
                todo!()
            }
            JudgeOp::ConfirmRound(r_id) => {
                todo!()
            }
        }
    }

    pub(crate) fn requires(&self, id: TournOfficialId) -> OpGroup {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => todo!(),
        }
    }
}
