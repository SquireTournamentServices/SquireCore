use squire_sdk::{
    model::rounds::{RoundId, RoundResult},
    players::{Player, PlayerId, Round},
};
use std::{collections::HashMap, fmt::Display, marker::PhantomData, rc::Rc, str::FromStr};
use yew::prelude::*;

use super::{RoundResultTicker, RoundResultTickerMessage};

#[derive(Debug, PartialEq, Clone)]
/// Data buffer holding changes which can be pushed to a round using admin operations
pub struct RoundChangesBuffer {
    pub rid: RoundId,
    pub draw_ticker: RoundResultTicker,
    pub win_tickers: HashMap<PlayerId, RoundResultTicker>,
}

#[derive(Debug, PartialEq, Clone)]
/// Message recieved by the data buffer
pub enum RoundChangesBufferMessage {
    TickClicked(Option<PlayerId>, RoundResultTickerMessage),
    ResetAll(),
}

impl RoundChangesBuffer {
    pub fn new(rid: RoundId, draw_ticker: RoundResultTicker) -> Self {
        Self {
            rid,
            draw_ticker,
            win_tickers: HashMap::new(),
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
        self.win_tickers.get(&pid).unwrap().view()
    }

    /// Draw the round's draw count with buttons to increment and decrement the value
    pub fn view_draw_ticker(&self) -> Html {
        self.draw_ticker.view()
    }
}
