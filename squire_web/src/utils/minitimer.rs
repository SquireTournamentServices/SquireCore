use std::time::Duration;

use squire_sdk::{players::Round, model::rounds::RoundId};
use chrono::{DateTime, Utc};
use yew::{prelude::*, props};

use crate::tournament::rounds::{RoundsView, RoundsFilterMessage};

pub struct Minitimer {
    pub rnd: Round,
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct MinitimerProps {
    pub rnd: Round,
}
pub enum MinitimerMessage {
    TickDown(RoundId),
    PopOut(),
}

impl Minitimer {

    pub fn time_left(&self) -> Duration {
        let length = self.rnd.length + self.rnd.extension;
        let elapsed = Duration::from_secs((Utc::now() - self.rnd.timer).num_seconds() as u64);
        if elapsed < length {
            length - elapsed
        } else {
            Duration::default()
        }
    }

    pub fn new(props: MinitimerProps) -> Self {
        let MinitimerProps { rnd } = props;
        Self {rnd}
    }

    pub fn update(&mut self, msg: MinitimerMessage) -> bool {
        match msg {
            MinitimerMessage::TickDown(r_id) => {
                r_id == self.rnd.id
            }
            MinitimerMessage::PopOut() => {
                // TODO
                false
            }
        }
    }

    pub fn view(&self, ctx: &Context<RoundsView>) -> Html {
        let id = self.rnd.id;
        ctx.link().send_future(async move { async_std::task::sleep(std::time::Duration::from_secs(1)).await; (RoundsFilterMessage::TimerTick(id)) });
        html! {
            <>
            {
                format!("~ {} ~", self.time_left().as_secs())
            }
            </>
        }
    }
}