use std::borrow::Cow;

use squire_sdk::{
    model::{
        identifiers::AdminId,
        operations::JudgeOp,
        players::PlayerId,
        rounds::{RoundId, RoundResult},
    },
    tournaments::TournOp,
};
use yew::prelude::*;

use super::SelectedRoundMessage;
use crate::tournament::rounds::roundchangesbuffer::RoundChangesBufferMessage;

#[derive(Debug, PartialEq, Clone)]
/// Sub component storing/displaying a result and associated changes to that result
pub struct RoundResultTicker {
    pub label: Cow<'static, str>,
    pub pid: Option<PlayerId>,
    pub stored_result: RoundResult,
    pub was_changed: bool,
    pub process: Callback<SelectedRoundMessage>,
}

#[derive(Debug, PartialEq, Clone)]
/// Message recieved by the ticker
pub enum RoundResultTickerMessage {
    Increment,
    Decrement,
}

impl RoundResultTicker {
    pub fn new(
        label: Cow<'static, str>,
        pid: Option<PlayerId>,
        stored_result: RoundResult,
        process: Callback<SelectedRoundMessage>,
    ) -> Self {
        Self {
            label,
            pid,
            stored_result,
            was_changed: false,
            process,
        }
    }

    pub fn into_op(&self, admin_id: AdminId, rid: RoundId) -> Option<TournOp> {
        if !self.was_changed {
            return None;
        }
        Some(TournOp::JudgeOp(
            admin_id.clone().into(),
            JudgeOp::AdminRecordResult(rid, self.stored_result),
        ))
    }

    pub fn update(&mut self, msg: RoundResultTickerMessage) -> bool {
        match msg {
            RoundResultTickerMessage::Increment => {
                self.stored_result.inc_result();
                self.was_changed = true;
            }
            RoundResultTickerMessage::Decrement => {
                self.stored_result.dec_result();
                self.was_changed = true;
            } /*
              RoundResultTickerMessage::SetChanged(val) => {
                  self.was_changed = val;
              }
              */
        }
        true
    }

    pub fn view(&self) -> Html {
        let pid = self.pid.clone();
        let cb = self.process.clone();
        let up = move |_| {
            cb.emit(SelectedRoundMessage::BufferMessage(
                RoundChangesBufferMessage::TickClicked(pid, RoundResultTickerMessage::Increment),
            ));
        };
        let pid = self.pid.clone();
        let cb = self.process.clone();
        let down = move |_| {
            cb.emit(SelectedRoundMessage::BufferMessage(
                RoundChangesBufferMessage::TickClicked(pid, RoundResultTickerMessage::Decrement),
            ));
        };
        let stored_result_value = self.stored_result.get_result();
        html! {
            <>
                <>{format!( "{} {}", self.label, stored_result_value )}</>
                <button onclick={up}>{"+"}</button>
                <button onclick={down}>{"-"}</button>
            </>
        }
    }
}
