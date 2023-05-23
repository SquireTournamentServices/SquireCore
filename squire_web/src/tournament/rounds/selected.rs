use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use squire_sdk::{
    model::{
        identifiers::AdminId,
        operations::{AdminOp, JudgeOp},
        rounds::{Round, RoundId, RoundResult, RoundStatus},
        tournament::Tournament,
    },
    players::PlayerId,
    tournaments::{TournOp, TournamentId},
};
use yew::prelude::*;

use crate::{tournament::players::RoundProfile, utils::console_log, CLIENT};

use super::{
    roundchangesbuffer::{self, *},
    roundresultticker::*,
    RoundResultTicker, RoundsView, RoundsViewMessage, RoundConfirmationTicker, RoundConfirmationTickerMessage,
};

/// Message to be passed to the selected round
#[derive(Debug, PartialEq, Clone)]
pub enum SelectedRoundMessage {
    RoundSelected(RoundId),
    TimerTicked(RoundId),
    /// Optional because the lookup "may" fail
    RoundQueryReady(Option<RoundProfile>),
    BufferMessage(RoundChangesBufferMessage),
    PushChanges(RoundId),
    BulkConfirm(RoundId),
}

/// Sub-Component displaying round currently selected
pub struct SelectedRound {
    pub t_id: TournamentId,
    pub admin_id: AdminId,
    // draw_ticker: RoundResultTicker,
    /// The data from the tournament that is used to display the round
    pub round: Option<(RoundProfile, RoundUpdater)>,
    // round_changes_buffer: Option<RoundChangesBuffer>,
    pub process: Callback<SelectedRoundMessage>,
}

impl SelectedRound {
    pub fn new(ctx: &Context<RoundsView>, t_id: TournamentId, admin_id: AdminId) -> Self {
        send_ticker_future(Default::default(), ctx);
        Self {
            t_id,
            admin_id,
            round: None,
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
                self.round = data;
                true
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
                    self.requery(ctx, id, r_id)
                }
                false
            }
            SelectedRoundMessage::BufferMessage(msg) => {
                let Some((rnd, updater)) = self.round.as_mut() else { return false };
                if (updater.round_changes_buffer.is_some()) {
                    updater.round_changes_buffer.as_mut().unwrap().update(msg)
                } else {
                    false
                }
            }
            SelectedRoundMessage::PushChanges(rid) => {
                let rcb = self
                    .round
                    .as_ref()
                    .unwrap()
                    .1
                    .round_changes_buffer
                    .as_ref()
                    .unwrap();
                let mut ops = Vec::with_capacity(rcb.win_tickers.len()+1);

                if (rcb.draw_ticker.was_changed) {
                    ops.push(TournOp::JudgeOp(
                        self.admin_id.clone().into(),
                        JudgeOp::AdminRecordResult(rid, rcb.draw_ticker.stored_result.clone()),
                    ));
                }
                ops.extend(rcb.win_tickers.values().filter_map(|wt| {
                    wt.into_op(self.admin_id, rid)
                }));
                ops.extend(rcb.confirmation_tickers.values().filter_map(|ct| {
                    ct.into_op(self.admin_id, rid)
                }));

                CLIENT.get().unwrap().bulk_update(self.t_id, ops);
                false
            }
            SelectedRoundMessage::BulkConfirm(rid) => {
                /*
                let mut rcb = self
                    .round
                    .as_mut()
                    .unwrap()
                    .1
                    .round_changes_buffer
                    .as_mut()
                    .unwrap();
                rcb.confirmation_tickers.values_mut().for_each(|rct|{
                    if (!rct.pre_confirmed) {
                        rct.update(RoundConfirmationTickerMessage::Check);
                    }
                });
                self.update(ctx, SelectedRoundMessage::PushChanges(rid));
                */
                CLIENT.get().unwrap().update_tourn(self.t_id, TournOp::JudgeOp(self.admin_id.clone().into(),JudgeOp::ConfirmRound(rid)));
                false
            }
        }
    }

    fn requery(&self, ctx: &Context<RoundsView>, tid: TournamentId, r_id: RoundId) {
        ctx.link().send_future(async move {
            let data = CLIENT
                .get()
                .unwrap()
                .query_tourn(tid, move |t| {
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

    pub fn try_requery_existing(&self, ctx: &Context<RoundsView>) {
        if (self.round.is_some()) {
            let r_id = self.round.as_ref().unwrap().0.id;
            self.requery(ctx, self.t_id, r_id)
        }
    }

    pub fn view(&self) -> Html {
        let Some((rnd, updater)) = self.round.as_ref() else { return Html::default() };
        // Moved to rnd.view()
        html! {
            <div class="m-2">
            <>
            <> { rnd.view() } </> // The "data" half of the round's view
            <hr />
            <p>
            {
                updater.view(&self.round.as_ref().unwrap().0)
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

/// Called once every second; updates the timer
fn send_ticker_future(id: RoundId, ctx: &Context<RoundsView>) {
    ctx.link().send_future(async move {
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        RoundsViewMessage::SelectedRound(SelectedRoundMessage::TimerTicked(id))
    });
}

#[derive(Debug, PartialEq, Clone)]
/// Portion of the selected round panel used for updating values
pub struct RoundUpdater {
    /// Used to store changes to round
    round_changes_buffer: Option<RoundChangesBuffer>,
    /// Round id
    rid: RoundId,
    /// Used to send messages up
    process: Callback<SelectedRoundMessage>,
}

impl RoundUpdater {
    pub fn new(rnd: &RoundProfile, process: Callback<SelectedRoundMessage>) -> Self {
        let proc = process.clone();
        let mut rcb = RoundChangesBuffer::new(
            rnd.id,
            RoundResultTicker::new("Draws".into(), None, RoundResult::Draw(rnd.draws), proc),
        );
        let proc = process.clone();
        //rcb.win_tickers.insert(*r.0, ticker);
        rcb.win_tickers.extend(rnd.player_names.iter().map(|r| {
            let found_result = rnd.results.get(r.0).cloned().unwrap_or_default();
            (
                *r.0,
                RoundResultTicker::new(
                    format!("{} wins: ", r.1).into(),
                    Some(*r.0),
                    RoundResult::Wins(*r.0, found_result),
                    proc.clone(),
                ),
            )
        }));
        rcb.confirmation_tickers.extend(rnd.player_names.iter().map(|r| {
            let found_result = rnd.results.get(r.0).cloned().unwrap_or_default();
            (
                *r.0,
                RoundConfirmationTicker::new(
                    rnd.confirmations.contains(r.0),
                    proc.clone(),
                    *r.0,
                ),
            )
        }));
        Self {
            round_changes_buffer: Some(rcb),
            rid: rnd.id,
            process,
        }
    }

    pub fn view(&self, rnd: &RoundProfile) -> Html {
        let rid = self.rid.clone();
        let mut cb = self.process.clone();
        let pushdata = move |me: MouseEvent| {
            cb.emit(SelectedRoundMessage::PushChanges(rid));
        };
        cb = self.process.clone();
        let bulkconfirm = move |me: MouseEvent| {
            cb.emit(SelectedRoundMessage::BulkConfirm(rid));
        };
        let win_list = rnd.order
            .iter()
            .map(|(pid)| {
                self.round_changes_buffer.as_ref().unwrap().view_win_ticker(*pid)
            })
            .collect::<Html>();
        let bulk_confirmed_disabled = rnd.status != RoundStatus::Open;
        html! {
            <>
            <p>{
                win_list
            }</p>
            <p>{
                self.round_changes_buffer.as_ref().unwrap().view_draw_ticker()
            }</p>
            <br />
            <button onclick={pushdata}>{"Submit changes"}</button>
            <button onclick={bulkconfirm} disabled={bulk_confirmed_disabled}>{"Bulk Confirm"}</button>
            </>
        }
    }
}
