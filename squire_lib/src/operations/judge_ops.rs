use std::time::Duration;

use serde::{Serialize, Deserialize};

use crate::{identifiers::{RoundId, PlayerId}, players::Deck, rounds::RoundResult, accounts::SquireAccount};


/// Operations that judges and tournament admin can perform
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum JudgeOp {
    /// Operation for adding a guest player to a tournament (i.e. someone without an account)
    RegisterGuest(String),
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
}
