use std::{collections::HashMap};

use chrono::{Duration, Utc};
use squire_sdk::{
    model::{
        rounds::{Round, RoundId, RoundResult, RoundStatus},
        tournament::Tournament,
    },
    players::PlayerId,
    tournaments::TournamentId,
};
use yew::prelude::*;

use crate::{tournament::players::RoundProfile, utils::console_log, CLIENT};

use super::{
    roundchangesbuffer::{*, self}, roundresultticker::*, RoundResultTicker, RoundsView, RoundsViewMessage,
};

#[derive(Debug, PartialEq, Clone)]
pub enum SelectedRoundMessage {
    RoundSelected(RoundId),
    TimerTicked(RoundId),
    /// Optional because the lookup "may" fail
    RoundQueryReady(Option<RoundProfile>),
    BufferMessage(RoundChangesBufferMessage),
}

pub struct SelectedRound {
    pub t_id: TournamentId,
    // draw_ticker: RoundResultTicker,
    /// The data from the tournament that is used to display the round
    round: Option<(RoundProfile, RoundUpdater)>,
    // round_changes_buffer: Option<RoundChangesBuffer>,
    pub process: Callback<SelectedRoundMessage>,
}

impl SelectedRound {
    pub fn new(ctx: &Context<RoundsView>, t_id: TournamentId) -> Self {
        send_ticker_future(Default::default(), ctx);
        Self {
            t_id,
            round: None,
            /*
            draw_ticker: RoundResultTicker::new(
                "Draws",
                None,
                RoundResult::Draw(0),
                ctx.link().callback(RoundsViewMessage::SelectedRound),
            ),
            round_changes_buffer: None,
            */
            process: ctx.link().callback(RoundsViewMessage::SelectedRound),
        }
    }

    pub fn update(&mut self, ctx: &Context<RoundsView>, msg: SelectedRoundMessage) -> bool {
        match msg {
            SelectedRoundMessage::TimerTicked(r_id) => match self.round.as_ref() {
                Some((rnd, _)) => {
                    let digest = rnd.id == r_id;
                    if digest {
                        send_ticker_future(r_id, ctx);
                    }
                    digest
                }
                None => {
                    send_ticker_future(Default::default(), ctx);
                    false
                }
            },
            SelectedRoundMessage::RoundQueryReady(rnd) => {
                let data = rnd.map(|rnd| {
                    send_ticker_future(rnd.id, ctx);
                    let updater = RoundUpdater::new(&rnd, self.process.clone());
                    (rnd, updater)
                });
                let digest = self.round != data;
                self.round = data;
                digest
            }
            SelectedRoundMessage::RoundSelected(r_id) => {
                console_log(&format!("Round selected: {r_id}"));
                if self
                    .round
                    .as_ref()
                    .map(|(r, _)| r.id != r_id)
                    .unwrap_or(true)
                {
                    let id = self.t_id;
                    ctx.link().send_future(async move {
                        let data = CLIENT
                            .get()
                            .unwrap()
                            .query_tourn(id, move |t| {
                                let tourn = t.tourn();
                                tourn
                                    .round_reg
                                    .get_round(&r_id)
                                    .map(|r| RoundProfile::new(tourn, r))
                            })
                            .process()
                            .await
                            .transpose()
                            .ok()
                            .flatten();
                        console_log(&format!("Round was found: {}", data.is_some()));
                        RoundsViewMessage::SelectedRound(SelectedRoundMessage::RoundQueryReady(
                            data,
                        ))
                    });
                }
                false
            }
            SelectedRoundMessage::BufferMessage(msg) => {
                let Some((rnd, updater)) = self.round.as_mut() else { return false };
                if (updater.round_changes_buffer.is_some()) {
                    updater.round_changes_buffer.as_mut().unwrap().update(msg)
                }
                else {
                    false
                }
            }
        }
    }

    pub fn view(&self) -> Html {
        let Some((rnd, updater)) = self.round.as_ref() else { return Html::default() };
        // Moved to rnd.view()
        html! {
            <div class="m-2">
            <>
            <> { rnd.view() } </> // The "data" half of the round's view
            <p>
            {
                updater.view()
                //self.round_changes_buffer.as_ref().unwrap().view_draw_ticker()
            }
            </p>
            // Moved to RoundUpdater's view method
            //<br/>
            //<button onclick={pushdata}>{"Submit changes"}</button>
            </>
            </div>
        }
    }
}

fn send_ticker_future(id: RoundId, ctx: &Context<RoundsView>) {
    ctx.link().send_future(async move {
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        RoundsViewMessage::SelectedRound(SelectedRoundMessage::TimerTicked(id))
    });
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoundUpdater {
    /// Used to store changes to round
    round_changes_buffer: Option<RoundChangesBuffer>,
}

impl RoundUpdater {
    pub fn new(rnd: &RoundProfile, process: Callback<SelectedRoundMessage>) -> Self {
        Self {
            round_changes_buffer: Some(RoundChangesBuffer::new(
                rnd.id,
                RoundResultTicker::new(
                    "Draw",
                    None,
                    RoundResult::Draw(rnd.draws),
                    process
                ),
            ))
        }
    }

    pub fn view(&self) -> Html {
        html! {
            <>
                <p>{
                    self.round_changes_buffer.as_ref().unwrap().view_draw_ticker()
                }</p>
            </>
        }
    }
}
