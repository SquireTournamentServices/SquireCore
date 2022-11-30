use serde::{Deserialize, Serialize};

use crate::{players::Deck, rounds::RoundResult, identifiers::PlayerId};

use super::OpGroup;

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
/// Operations that players can perform
pub enum PlayerOp {
    /// Operation for a player check themself into a tournament
    CheckIn,
    /// Operation for a player drop themself from a tournament
    DropPlayer,
    /// Operation for a player record their round result
    RecordResult(RoundResult),
    /// Operation for a player confirm their round result
    ConfirmResult,
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
    pub(crate) fn affects(&self, id: PlayerId) -> OpGroup {
        todo!()
    }
    
    pub(crate) fn requires(&self, id: PlayerId) -> OpGroup {
        todo!()
    }
}
