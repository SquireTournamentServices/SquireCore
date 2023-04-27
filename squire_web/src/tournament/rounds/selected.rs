use std::collections::HashMap;

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

use crate::{tournament::players::RoundProfile, CLIENT};

use super::{
    roundchangesbuffer::*, roundresultticker::*, RoundResultTicker, RoundsView, RoundsViewMessage,
};

pub fn round_info_display(rnd: &Round) -> Html {
    html! {
        <>
            <p>{ format!("Round #{} at table #{}", rnd.match_number, rnd.table_number) }</p>
            <p>{ format!("Status : {}", rnd.status) }</p>
            <p>{ format!("Bye : {}", rnd.is_bye()) }</p>
        </>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectedRoundMessage {
    RoundSelected(RoundId),
    TimerTicked(RoundId),
    RoundQueryReady(Option<RoundProfile>), // Optional because the lookup "may" fail
    BufferMessage(RoundChangesBufferMessage),
}

pub struct SelectedRound {
    pub t_id: TournamentId,
    round: Option<RoundProfile>,
    draw_ticker: RoundResultTicker,
    round_data_buffer: Option<Round>,
    round_changes_buffer: Option<RoundChangesBuffer>,
    pub process: Callback<SelectedRoundMessage>,
}

impl SelectedRound {
    pub fn new(ctx: &Context<RoundsView>, t_id: TournamentId) -> Self {
        send_ticker_future(Default::default(), ctx);
        Self {
            t_id,
            round_data_buffer: None,
            round: None,
            draw_ticker: RoundResultTicker::new(
                "Draws",
                None,
                RoundResult::Draw(0),
                ctx.link().callback(RoundsViewMessage::SelectedRound),
            ),
            round_changes_buffer: None,
            process: ctx.link().callback(RoundsViewMessage::SelectedRound),
        }
    }

    pub fn update(&mut self, ctx: &Context<RoundsView>, msg: SelectedRoundMessage) -> bool {
        match msg {
            SelectedRoundMessage::TimerTicked(r_id) => match self.round.as_ref() {
                Some(rnd) => {
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
                let digest = self.round != rnd;
                self.round = rnd;
                digest
            }
            SelectedRoundMessage::RoundSelected(r_id) => {
                if self.round.as_ref().map(|r| r.id != r_id).unwrap_or(true) {
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
                        RoundsViewMessage::SelectedRound(SelectedRoundMessage::RoundQueryReady(
                            data,
                        ))
                    });
                }
                false
            }
            SelectedRoundMessage::BufferMessage(msg) => {
                match msg {
                    RoundChangesBufferMessage::TickClicked(pid, msg) => {
                        todo!()
                    }
                    RoundChangesBufferMessage::ResetAll() => todo!(),
                }
            }
        }
    }

    pub fn view(&self) -> Html {
        let pushdata = move |_| {
            // TODO: Push the buffer data to the actual round in the database
            todo!();
        };
        let returnhtml = self.round_data_buffer.as_ref()
            .map(|rnd| {
                // TODO: Remove unwrap here
                let dur_left = Duration::from_std(rnd.length + rnd.extension).unwrap() - (Utc::now() - rnd.timer);
                html! {
                    <>
                    <>{round_info_display(rnd)}</>
                    <ul>
                    {
                        self.round.as_ref().map(|r| r.player_names.iter()
                            // Right now this code is duplicated, however once SelectedRound has more functionality it will be made significantly different. (It will have onclick functionality.)
                            .map(|(pid, name)| {
                                let player_wins = rnd.results.get(pid).cloned().unwrap_or_default();
                                let player_confirm = rnd.confirmations.get(pid).is_some();
                                html! {
                                    <li>
                                    <div>
                                    { format!( "{name}") }
                                    </div>
                                    <div>
                                    { format!( "wins : {player_wins}, confirmed : {player_confirm}") }
                                    </div>
                                    </li>
                                }
                            })
                            .collect::<Html>()).unwrap_or_default()
                    }
                    </ul>
                    <p>
                    {
                        self.round_changes_buffer.as_ref().unwrap().view_draw_ticker()
                    }
                    </p>
                    <p>
                    { pretty_print_duration(dur_left) }
                    </p>
                    <br/>
                    <button onclick={pushdata}>{"Submit changes"}</button>
                    </>
                }
            })
            .unwrap_or_else(|| html!{
                <h4>{"Round not found"}</h4>
            });
        html! {
            <div class="m-2">{returnhtml}</div>
        }
    }
}

fn pretty_print_duration(dur: Duration) -> String {
    let hours = dur.num_hours();
    let mins = dur.num_minutes();
    let secs = dur.num_seconds();
    if hours < 0 {
        format!("Time left: {hours}:{}:{}", mins.abs() % 60, secs.abs() % 60)
    } else {
        format!(
            "Over time: {}:{}:{}",
            hours.abs(),
            mins.abs() % 60,
            secs.abs() % 60
        )
    }
}

fn send_ticker_future(id: RoundId, ctx: &Context<RoundsView>) {
    ctx.link().send_future(async move {
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        RoundsViewMessage::SelectedRound(SelectedRoundMessage::TimerTicked(id))
    });
}
