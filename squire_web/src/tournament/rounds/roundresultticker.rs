use std::{marker::PhantomData, str::FromStr, rc::Rc, fmt::Display};
use squire_sdk::{model::rounds::RoundResult, players::{Round, PlayerId}};
use yew::prelude::*;

use crate::tournament::rounds::roundchangesbuffer::RoundChangesBufferMessage;

use super::SelectedRoundMessage;

pub struct RoundResultTicker {
    pub label: &'static str,
    pub pid: Option<PlayerId>,
    pub stored_result: RoundResult,
    pub was_changed: bool,
    pub process: Callback<SelectedRoundMessage>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RoundResultTickerMessage {
    Increment,
    Decrement,
    SetChanged(bool),
}

impl RoundResultTicker {
    pub fn new(
        label: &'static str,
        pid: Option<PlayerId>,
        stored_result: RoundResult,
        process: Callback<SelectedRoundMessage>
    ) -> Self
    {
        Self {
            label,
            pid,
            stored_result,
            was_changed: false,
            process,
        }
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
            }
            RoundResultTickerMessage::SetChanged(val) => {
                self.was_changed = val;
            }
        }
        true
    }

    #[allow(clippy::option_map_unit_fn)]
    pub fn view(&self) -> Html {
        let pid = self.pid.clone();
        let cb = self.process.clone();
        let up = move |s| {
            cb.emit(SelectedRoundMessage::BufferMessage(RoundChangesBufferMessage::TickClicked( pid, RoundResultTickerMessage::Increment )));
        };
        let pid = self.pid.clone();
        let cb = self.process.clone();
        let down = move |s| {
            cb.emit(SelectedRoundMessage::BufferMessage(RoundChangesBufferMessage::TickClicked( pid, RoundResultTickerMessage::Decrement )));
        };
        let label = self.label;
        let stored_result_value =  self.stored_result.get_result();
        html! {
            <>
                <>{format!( "{} {}", label, stored_result_value )}</>
                <button onclick={up}>{"+"}</button>
                <button onclick={down}>{"-"}</button>
            </>
        }
    }
}
