use squire_sdk::{
    model::{rounds::{RoundResult, RoundId}, identifiers::AdminId, operations::JudgeOp},
    players::{PlayerId, Round}, tournaments::TournOp,
};
use std::{fmt::Display, marker::PhantomData, rc::Rc, str::FromStr};
use yew::prelude::*;

use crate::tournament::rounds::{roundchangesbuffer::RoundChangesBufferMessage, RoundsViewMessage};

use super::SelectedRoundMessage;

use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
/// Sub component storing/displaying a result and associated changes to that result
pub struct RoundConfirmationTicker {
    pub process: Callback<SelectedRoundMessage>,
}

#[derive(Debug, PartialEq, Clone)]
/// Message recieved by the ticker
pub enum RoundConfirmationTickerMessage {
    Check,
    Uncheck,
}

impl RoundConfirmationTicker {
    pub fn new(
        process: Callback<SelectedRoundMessage>,
    ) -> Self {
        Self {
            process,
        }
    }

    pub fn into_op(&self, admin_id : AdminId, rid : RoundId) -> Option<TournOp> {
       todo!()
    }

    pub fn update(&mut self, msg: RoundConfirmationTickerMessage) -> bool {
        match msg {
            RoundConfirmationTickerMessage::Check => {
                todo!()
            }
            RoundConfirmationTickerMessage::Uncheck => {
                todo!()
            }
        }
        true
    }

    pub fn view(&self) -> Html {
       html!{
        <>{ "TODO" }</>
       }
    }
}
