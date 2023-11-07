use squire_sdk::model::{operations::JudgeOp, players::PlayerId, rounds::RoundId};
use yew::prelude::*;

use super::SelectedRoundMessage;
use crate::tournament::{rounds::roundchangesbuffer::RoundChangesBufferMessage, Op};

#[derive(Debug, PartialEq, Clone)]
/// Sub component storing/displaying a confirmation for a player
pub struct RoundConfirmationTicker {
    /// If this is set to true, the player was confirmed when the round was loaded and thus the ticker will be greyed out
    pub pre_confirmed: bool,
    /// If true, confirm player on submission.
    pub currently_confirmed: bool,
    /// Callback
    pub process: Callback<SelectedRoundMessage>,
    /// Player ID
    pub pid: PlayerId,
}

#[derive(Debug, PartialEq, Clone)]
/// Message recieved by the ticker
pub enum RoundConfirmationTickerMessage {
    Check,
    Uncheck,
}

impl RoundConfirmationTicker {
    pub fn new(
        pre_confirmed: bool,
        process: Callback<SelectedRoundMessage>,
        pid: PlayerId,
    ) -> Self {
        Self {
            pre_confirmed,
            currently_confirmed: pre_confirmed,
            process,
            pid,
        }
    }

    pub fn as_op(&self, rid: RoundId) -> Option<Op> {
        (self.currently_confirmed && !self.pre_confirmed)
            .then(|| Op::Judge(JudgeOp::AdminConfirmResult(rid, self.pid)))
    }

    pub fn update(&mut self, msg: RoundConfirmationTickerMessage) -> bool {
        match msg {
            RoundConfirmationTickerMessage::Check => {
                if !self.pre_confirmed {
                    self.currently_confirmed = true;
                    return true;
                }
                false
            }
            RoundConfirmationTickerMessage::Uncheck => {
                if !self.pre_confirmed {
                    self.currently_confirmed = false;
                    return true;
                }
                false
            }
        }
    }

    pub fn view(&self) -> Html {
        let mut pid = self.pid;
        let cb = self.process.clone();
        let pre_confirmed = self.pre_confirmed;
        let currently_confirmed = self.currently_confirmed;
        let check = move |_| {
            if currently_confirmed {
                cb.emit(SelectedRoundMessage::BufferMessage(
                    RoundChangesBufferMessage::ConfirmationClicked(
                        pid,
                        RoundConfirmationTickerMessage::Uncheck,
                    ),
                ));
            } else {
                cb.emit(SelectedRoundMessage::BufferMessage(
                    RoundChangesBufferMessage::ConfirmationClicked(
                        pid,
                        RoundConfirmationTickerMessage::Check,
                    ),
                ));
            }
        };
        pid = self.pid;
        html! {
            <input type={"checkbox"} id={pid.to_string()} disabled={pre_confirmed} checked={currently_confirmed} onclick={check} />
        }
    }
}
