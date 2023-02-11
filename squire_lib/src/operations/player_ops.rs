use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{identifiers::RoundId, operations::OpUpdate, players::Deck, rounds::RoundResult};

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
/// Operations that players can perform
pub enum PlayerOp {
    /// Operation for a player check themself into a tournament
    CheckIn,
    /// Operation for a player drop themself from a tournament
    DropPlayer,
    /// Operation for a player record their round result
    RecordResult(RoundId, RoundResult),
    /// Operation for a player confirm their round result
    ConfirmResult(RoundId),
    /// Operation for a player add a deck to their registration information
    AddDeck(String, Deck),
    /// Operation for a player remove a deck to their registration information
    RemoveDeck(String),
    /// Operation for a player set their gamer tag
    SetGamerTag(String),
    /// Operation for a player to mark themself as ready for their next round
    ReadyPlayer,
    /// Operation for a player to mark themself as unready for their next round
    UnReadyPlayer,
}

impl PlayerOp {
    pub(crate) fn get_update(&self, _salt: DateTime<Utc>) -> OpUpdate {
        OpUpdate::None
    }

    pub(crate) fn swap_round_ids(&mut self, old: RoundId, new: RoundId) {
        match self {
            PlayerOp::RecordResult(r_id, _) | PlayerOp::ConfirmResult(r_id) if *r_id == old => {
                *r_id = new;
            }
            _ => {}
        }
    }
}
