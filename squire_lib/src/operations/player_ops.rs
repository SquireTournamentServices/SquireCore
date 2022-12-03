use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{
    identifiers::{PlayerId, RoundId},
    players::Deck,
    rounds::RoundResult,
};

use super::{
    OpEffects, OpGroup, OpPlayerEffects, OpRoundEffects, PlayerEffectComponent,
    RoundEffectComponent,
};

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
    pub(crate) fn affects(&self, id: PlayerId) -> OpGroup {
        match self {
            PlayerOp::CheckIn => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::CheckIn,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::DropPlayer => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Status,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::RecordResult(r_id, result) => {
                let p_id = match result {
                    RoundResult::Wins(id, _) => Some(*id),
                    RoundResult::Draw(..) => None,
                };
                let effects = Cow::Owned(vec![
                    OpEffects::Player(OpPlayerEffects::SingleActive(
                        id,
                        PlayerEffectComponent::Nothing,
                    )),
                    OpEffects::Round(OpRoundEffects::SingleActive(
                        *r_id,
                        RoundEffectComponent::Result(p_id),
                    )),
                    OpEffects::Round(OpRoundEffects::SingleActive(
                        *r_id,
                        RoundEffectComponent::Confirmation,
                    )),
                ]);
                OpGroup { effects }
            }
            PlayerOp::ConfirmResult(r_id) => {
                let effects = Cow::Owned(vec![
                    OpEffects::Player(OpPlayerEffects::SingleActive(
                        id,
                        PlayerEffectComponent::Nothing,
                    )),
                    OpEffects::Round(OpRoundEffects::SingleActive(
                        *r_id,
                        RoundEffectComponent::Confirmation,
                    )),
                ]);
                OpGroup { effects }
            }
            PlayerOp::AddDeck(name, deck) => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Deck(name.clone()),
                ))]);
                OpGroup { effects }
            }
            PlayerOp::RemoveDeck(name) => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Deck(name.clone()),
                ))]);
                OpGroup { effects }
            }
            PlayerOp::SetGamerTag(name) => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Nothing,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::ReadyPlayer => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::CheckIn,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::UnReadyPlayer => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::CheckIn,
                ))]);
                OpGroup { effects }
            }
        }
    }

    pub(crate) fn requires(&self, id: PlayerId) -> OpGroup {
        match self {
            PlayerOp::CheckIn => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Nothing,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::DropPlayer => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Nothing,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::RecordResult(r_id, result) => {
                let p_id = match result {
                    RoundResult::Wins(id, _) => Some(*id),
                    RoundResult::Draw(..) => None,
                };
                let effects = Cow::Owned(vec![
                    OpEffects::Player(OpPlayerEffects::SingleActive(
                        id,
                        PlayerEffectComponent::Nothing,
                    )),
                    OpEffects::Round(OpRoundEffects::SingleActive(
                        *r_id,
                        RoundEffectComponent::Nothing,
                    )),
                ]);
                OpGroup { effects }
            }
            PlayerOp::ConfirmResult(r_id) => {
                let effects = Cow::Owned(vec![
                    OpEffects::Player(OpPlayerEffects::SingleActive(
                        id,
                        PlayerEffectComponent::Nothing,
                    )),
                    OpEffects::Round(OpRoundEffects::SingleActive(
                        *r_id,
                        RoundEffectComponent::Nothing,
                    )),
                ]);
                OpGroup { effects }
            }
            PlayerOp::AddDeck(name, deck) => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Nothing,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::RemoveDeck(name) => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Deck(name.clone()),
                ))]);
                OpGroup { effects }
            }
            PlayerOp::SetGamerTag(name) => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Nothing,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::ReadyPlayer => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::Nothing,
                ))]);
                OpGroup { effects }
            }
            PlayerOp::UnReadyPlayer => {
                let effects = Cow::Owned(vec![OpEffects::Player(OpPlayerEffects::SingleActive(
                    id,
                    PlayerEffectComponent::CheckIn,
                ))]);
                OpGroup { effects }
            }
        }
    }
}
