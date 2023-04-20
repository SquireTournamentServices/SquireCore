use std::{marker::PhantomData, str::FromStr, rc::Rc, fmt::Display, collections::HashMap};
use squire_sdk::{model::rounds::{RoundResult, RoundId}, players::{Round, PlayerId, Player}};
use yew::prelude::*;

use super::{RoundResultTickerMessage, RoundResultTicker};

pub struct RoundChangesBuffer {
    pub rid : RoundId,
    pub draw_ticker : RoundResultTicker,
    pub win_tickers : HashMap<PlayerId, RoundResultTicker>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RoundChangesBufferMessage {
    TickClicked(Option<PlayerId>, RoundResultTickerMessage),
    ResetAll(),
}

impl RoundChangesBuffer {

    pub fn new(
        rid : RoundId,
        draw_ticker : RoundResultTicker,
    ) -> Self
    {
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
                // TODO loop through set all win tickers to be unchanged
                todo!();
            }
        }
        true
    }

    pub fn view_win_ticker(&self, pid: PlayerId) -> Html {
        self.win_tickers.get(&pid).unwrap().view()
    }

    pub fn view_draw_ticker(&self) -> Html {
        self.draw_ticker.view()
    }

}