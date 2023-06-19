use std::{collections::HashMap, fmt::Display, marker::PhantomData, rc::Rc, str::FromStr};

use squire_sdk::model::{
    players::{Player, PlayerId},
    rounds::{Round, RoundId, RoundResult},
};
use yew::prelude::*;

use super::{
    RoundConfirmationTicker, RoundConfirmationTickerMessage, RoundResultTicker,
    RoundResultTickerMessage, SelectedRoundMessage,
};

#[derive(Debug, PartialEq, Clone)]
/// Data buffer holding changes which can be pushed to a round using admin operations
pub struct RoundChangesBuffer {
    pub rid: RoundId,
    pub draw_ticker: RoundResultTicker,
    pub win_tickers: HashMap<PlayerId, RoundResultTicker>,
    pub confirmation_tickers: HashMap<PlayerId, RoundConfirmationTicker>,
    pub current_extension_minutes: u64,
    pub process: Callback<SelectedRoundMessage>,
}

#[derive(Debug, PartialEq, Clone)]
/// Message recieved by the data buffer
pub enum RoundChangesBufferMessage {
    TickClicked(Option<PlayerId>, RoundResultTickerMessage),
    ConfirmationClicked(PlayerId, RoundConfirmationTickerMessage),
    ExtensionIncrease(),
    ExtensionDecrease(),
    ResetAll(),
}

impl RoundChangesBuffer {
    pub fn new(
        process: Callback<SelectedRoundMessage>,
        rid: RoundId,
        draw_ticker: RoundResultTicker,
    ) -> Self {
        Self {
            rid,
            draw_ticker,
            win_tickers: HashMap::new(),
            confirmation_tickers: HashMap::new(),
            current_extension_minutes: 0,
            process,
        }
    }

    pub fn update(&mut self, msg: RoundChangesBufferMessage) -> bool {
        match msg {
            RoundChangesBufferMessage::TickClicked(pid, tmsg) => {
                if let Some(pid) = pid {
                    self.win_tickers.get_mut(&pid).unwrap().update(tmsg);
                } else {
                    self.draw_ticker.update(tmsg);
                }
            }
            RoundChangesBufferMessage::ConfirmationClicked(pid, rctmsg) => {
                self.confirmation_tickers
                    .get_mut(&pid)
                    .unwrap()
                    .update(rctmsg);
            }
            RoundChangesBufferMessage::ExtensionIncrease() => {
                self.current_extension_minutes += 1;
            }
            RoundChangesBufferMessage::ExtensionDecrease() => {
                self.current_extension_minutes = self.current_extension_minutes.saturating_sub(1);
            }
            RoundChangesBufferMessage::ResetAll() => {
                self.draw_ticker
                    .update(RoundResultTickerMessage::SetChanged(false));
                self.win_tickers.iter_mut().for_each(|(pid, wt)| {
                    wt.update(RoundResultTickerMessage::SetChanged(false));
                });
            }
        }
        true
    }

    /// Given a player's id, draw the player's win results with buttons to increment and decrement the value
    pub fn view_win_ticker(&self, pid: PlayerId) -> Html {
        html! {
            <p>
            <>{ self.win_tickers.get(&pid).unwrap().view() }</>
            <>{ self.confirmation_tickers.get(&pid).unwrap().view() }</>
            </p>
        }
    }

    /// Draw the round's draw count with buttons to increment and decrement the value
    pub fn view_draw_ticker(&self) -> Html {
        self.draw_ticker.view()
    }

    /// View extension ticker
    pub fn view_extension_ticker(&self) -> Html {
        let cb = self.process.clone();
        let up = move |s| {
            cb.emit(SelectedRoundMessage::BufferMessage(
                RoundChangesBufferMessage::ExtensionIncrease(),
            ));
        };
        let cb = self.process.clone();
        let down = move |s| {
            cb.emit(SelectedRoundMessage::BufferMessage(
                RoundChangesBufferMessage::ExtensionDecrease(),
            ));
        };
        html! {
            <>
            <>{ format!("Time to extend in minutes : {}", self.current_extension_minutes.to_string()) }</>
            <button onclick={up}>{"+"}</button>
            <button onclick={down}>{"-"}</button>
            </>
        }
    }
}
