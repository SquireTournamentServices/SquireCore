use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use squire_sdk::{
    model::{
        identifiers::AdminId,
        operations::{AdminOp, JudgeOp},
        players::PlayerId,
        rounds::{Round, RoundId, RoundResult, RoundStatus},
        tournament::Tournament,
    },
    tournaments::{OpResult, TournOp, TournamentId},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use super::{
    roundchangesbuffer::{self, *},
    roundresultticker::*,
    RoundConfirmationTicker, RoundConfirmationTickerMessage, RoundResultTicker, RoundsView,
    RoundsViewMessage,
};
use crate::{utils::console_log, CLIENT, tournament::players::PlayerProfile};

/// The set of data needed by the UI to display a round. Should be capable of rendering itself in
/// HTML.
///
/// NOTE: Under construction
#[derive(Debug, PartialEq, Clone)]
pub struct RoundProfile {
    pub id: RoundId,
    pub order: Vec<PlayerId>,
    pub player_names: HashMap<PlayerId, String>,
    pub timer: DateTime<Utc>,
    pub status: RoundStatus,
    pub results: HashMap<PlayerId, u32>,
    pub draws: u32,
    pub confirmations: HashSet<PlayerId>,
    pub length: std::time::Duration,
    pub extensions: std::time::Duration,
}
impl RoundProfile {
    pub fn new(tourn: &Tournament, rnd: &Round) -> Self {
        Self {
            id: rnd.id,
            status: rnd.status,
            order: rnd.players.clone(),
            player_names: rnd
                .players
                .iter()
                .filter_map(|p| {
                    tourn
                        .player_reg
                        .players
                        .get(p)
                        .map(|plyr| (*p, plyr.name.clone()))
                })
                .collect(), // This is not a Vec<(PlayerId, String)>. This is a HashMap
            length: rnd.length,
            extensions: rnd.extension,
            timer: rnd.timer,
            results: rnd.results.clone(),
            draws: rnd.draws,
            confirmations: rnd.confirmations.clone(),
        }
    }

    pub fn view(&self) -> Html {
        // TODO: Remove unwrap here
        let dur_left = ChronoDuration::from_std(self.length + self.extensions).unwrap()
            - (Utc::now() - self.timer);
        let list = self
            .order
            .iter()
            .map(|pid| {
                    let player_name = self.player_names.get(pid).cloned().unwrap_or_default();
                    let player_wins = self.results.get(pid).cloned().unwrap_or_default();
                    let player_confirm = self.confirmations.get(pid).is_some();
                    html! {
                        <tr>
                            <td>{ player_name }</td>
                            <td>{ player_wins }</td>
                            <td>{ player_confirm }</td>
                        </tr>
                    }
            })
            .collect::<Html>();
        html! {
            <>
            <p>
            { pretty_print_duration(dur_left) }
            </p>
            <table class="table">
            <thead>
                <tr>
                    <th>{ "Name" }</th>
                    <th>{ "Wins" }</th>
                    <th>{ "Status" }</th>
                </tr>
            </thead>
            <tbody> { list } </tbody>
            </table>
            </>
        }
    }
}

/// Message to be passed to the selected round
#[derive(Debug, PartialEq, Clone)]
pub enum SelectedRoundMessage {
    RoundSelected(RoundId),
    PlayerSelected(PlayerId),
    TimerTicked(RoundId),
    /// Optional because the lookup "may" fail
    RoundQueryReady(Option<RoundProfile>),
    PlayerQueryReady(Option<PlayerProfile>),
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
            SelectedRoundMessage::PlayerSelected(p_id) => {
                false
            }
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
            SelectedRoundMessage::PlayerQueryReady(p_prof) => {
                false
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
                let Some((rnd, updater)) = self.round.as_mut() else {
                    return false;
                };
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
                let mut ops = Vec::with_capacity(rcb.win_tickers.len() + 1);

                if (rcb.draw_ticker.was_changed) {
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
                if (rcb.current_extension_minutes > 0) {
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
                spawn_local(async move {
                    console_log("Waiting for update to finish!");
                    send_op_result.emit(tracker.process().await.unwrap())
                });
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
            console_log(&format!("Round was found: {}", data.is_some()));
            RoundsViewMessage::SelectedRound(SelectedRoundMessage::RoundQueryReady(data))
        });
    }

    pub fn try_requery_existing(&self, ctx: &Context<RoundsView>) {
        if (self.round.is_some()) {
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
        cb = self.process.clone();
        let killround = move |me: MouseEvent| {
            cb.emit(SelectedRoundMessage::KillRound(rid));
        };
        let win_list = rnd
            .order
            .iter()
            .map(|(pid)| {
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

fn pretty_print_duration(dur: ChronoDuration) -> String {
    let hours = dur.num_hours();
    let mins = dur.num_minutes().abs();
    let secs = dur.num_seconds().abs();
    if hours >= 0 {
        format!("Time left: {}:{}:{}", hours.abs(), mins % 60, secs % 60)
    } else {
        format!("Over time: {}:{}:{}", hours.abs(), mins % 60, secs % 60)
    }
}

fn round_description_table(rnd: RoundProfile) -> Html {
    html! {
        <>{ "Nope" }</>
    }
}