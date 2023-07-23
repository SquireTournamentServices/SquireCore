use std::time::Duration;

use squire_sdk::model::{
    identifiers::{AdminId, TournamentId},
    operations::{AdminOp, JudgeOp, OpResult, TournOp},
    rounds::{RoundId, RoundResult, RoundStatus},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use super::{
    roundchangesbuffer::*, RoundConfirmationTicker, RoundResultTicker, RoundsView,
    RoundsViewMessage,
};
use crate::{tournament::model::RoundProfile, CLIENT};

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
    KillRound(RoundId),
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

    pub fn update(
        &mut self,
        ctx: &Context<RoundsView>,
        msg: SelectedRoundMessage,
        send_op_result: &Callback<OpResult>,
    ) -> bool {
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
                let Some((_rnd, updater)) = self.round.as_mut() else {
                    return false;
                };
                if updater.round_changes_buffer.is_some() {
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
                let mut ops = Vec::with_capacity(rcb.win_tickers.len() + 1);

                if rcb.draw_ticker.was_changed {
                    ops.push(TournOp::JudgeOp(
                        self.admin_id.clone().into(),
                        JudgeOp::AdminRecordResult(rid, rcb.draw_ticker.stored_result.clone()),
                    ));
                }
                ops.extend(
                    rcb.win_tickers
                        .values()
                        .filter_map(|wt| wt.into_op(self.admin_id, rid)),
                );
                ops.extend(
                    rcb.confirmation_tickers
                        .values()
                        .filter_map(|ct| ct.into_op(self.admin_id, rid)),
                );
                if rcb.current_extension_minutes > 0 {
                    ops.push(TournOp::JudgeOp(
                        self.admin_id.clone().into(),
                        JudgeOp::TimeExtension(
                            rid,
                            Duration::from_secs(rcb.current_extension_minutes * 60),
                        ),
                    ));
                }

                // Update methods return a tracker that is a future and needs to be awaited
                let tracker = CLIENT.get().unwrap().bulk_update(self.t_id, ops);
                let send_op_result = send_op_result.clone();
                spawn_local(async move { send_op_result.emit(tracker.process().await.unwrap()) });
                false
            }
            SelectedRoundMessage::BulkConfirm(rid) => {
                CLIENT.get().unwrap().update_tourn(
                    self.t_id,
                    TournOp::JudgeOp(self.admin_id.clone().into(), JudgeOp::ConfirmRound(rid)),
                );
                false
            }
            SelectedRoundMessage::KillRound(rid) => {
                CLIENT.get().unwrap().update_tourn(
                    self.t_id,
                    TournOp::AdminOp(self.admin_id.clone().into(), AdminOp::RemoveRound(rid)),
                );
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
            RoundsViewMessage::SelectedRound(SelectedRoundMessage::RoundQueryReady(data))
        });
    }

    pub fn try_requery_existing(&self, ctx: &Context<RoundsView>) {
        if self.round.is_some() {
            let r_id = self.round.as_ref().unwrap().0.id;
            self.requery(ctx, self.t_id, r_id)
        }
    }

    pub fn view(&self) -> Html {
        let Some((rnd, updater)) = self.round.as_ref() else {
            return Html::default();
        };
        // Moved to rnd.view()
        html! {
            <div class="m-2">
            <>
            <> { rnd.view() } </>
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
        let mut proc = process.clone();
        let mut rcb = RoundChangesBuffer::new(
            proc.clone(),
            rnd.id,
            RoundResultTicker::new("Draws".into(), None, RoundResult::Draw(rnd.draws), proc),
        );
        proc = process.clone();
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
        rcb.confirmation_tickers
            .extend(rnd.player_names.iter().map(|r| {
                // let found_result = rnd.results.get(r.0).cloned().unwrap_or_default();
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
        let pushdata = move |_| {
            cb.emit(SelectedRoundMessage::PushChanges(rid));
        };
        cb = self.process.clone();
        let bulkconfirm = move |_| {
            cb.emit(SelectedRoundMessage::BulkConfirm(rid));
        };
        cb = self.process.clone();
        let killround = move |_| {
            cb.emit(SelectedRoundMessage::KillRound(rid));
        };
        let win_list = rnd
            .order
            .iter()
            .map(|pid| {
                self.round_changes_buffer
                    .as_ref()
                    .unwrap()
                    .view_win_ticker(*pid)
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
            <p>{
                self.round_changes_buffer.as_ref().unwrap().view_extension_ticker()
            }</p>
            <br />
            <button onclick={pushdata}>{"Submit changes"}</button>
            <button onclick={bulkconfirm} disabled={bulk_confirmed_disabled}>{"Bulk Confirm"}</button>
            <br />
            <button type="button" class="btn btn-danger" data-bs-toggle="modal" data-bs-target="#killModal">
            {"Kill round ☠️"}
            </button>
            <div class="modal fade" id="killModal" tabindex="-1" aria-labelledby="killModalLabel" aria-hidden="true">
              <div class="modal-dialog">
                <div class="modal-content">
                  <div class="modal-header">
                    <h1 class="modal-title fs-5" id="exampleModalLabel">{"Kill round confirmation"}</h1>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                  </div>
                  <div class="modal-body">
                    {"Do you REALLY want to kill the round?"}
                  </div>
                  <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Go back"}</button>
                    <button type="button" onclick={killround} class="btn btn-primary" data-bs-dismiss="modal">{"Kill round"}</button>
                  </div>
                </div>
              </div>
            </div>
            </>
        }
    }
}
