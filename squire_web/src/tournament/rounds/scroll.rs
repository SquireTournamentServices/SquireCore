use squire_sdk::model::{
    identifiers::TournamentId,
    rounds::{Round, RoundId, RoundStatus},
};
use yew::prelude::*;

use super::{input::RoundFilterReport, RoundsView, RoundsViewMessage, SelectedRoundMessage};
use crate::CLIENT;

#[derive(Debug, PartialEq, Clone)]
pub enum RoundScrollMessage {
    ScrollQueryReady(Vec<RoundSummary>),
}

pub struct RoundScroll {
    pub id: TournamentId,
    pub process: Callback<RoundId>,
    rounds: Vec<RoundSummary>,
}

fn fetch_round_summaries(ctx: &Context<RoundsView>, id: TournamentId) {
    ctx.link().send_future(async move {
        let mut data = CLIENT
            .get()
            .unwrap()
            .query_rounds(id, |rnds| {
                rnds.rounds
                    .values()
                    .map(RoundSummary::new)
                    .collect::<Vec<_>>()
            })
            .await
            .unwrap_or_default();
        data.sort_by_cached_key(|r| r.match_number);
        data.sort_by_cached_key(|r| r.status);
        RoundsViewMessage::RoundScroll(RoundScrollMessage::ScrollQueryReady(data))
    })
}

impl RoundScroll {
    pub fn new(ctx: &Context<RoundsView>, id: TournamentId) -> Self {
        fetch_round_summaries(ctx, id);
        Self {
            id,
            process: ctx.link().callback(SelectedRoundMessage::RoundSelected),
            rounds: Default::default(),
        }
    }

    pub fn requery(&self, ctx: &Context<RoundsView>) {
        fetch_round_summaries(ctx, self.id);
    }

    pub fn update(&mut self, msg: RoundScrollMessage) -> bool {
        match msg {
            RoundScrollMessage::ScrollQueryReady(rounds) => {
                let digest = self.rounds != rounds;
                self.rounds = rounds;
                digest
            }
        }
    }

    pub fn view(&self, report: RoundFilterReport) -> Html {
        let list = self
            .rounds
            .iter()
            .cloned()
            .filter_map(|r| {
                report.matches(&r).then(|| {
                    let cb = self.process.clone();
                    html! {
                        <tr onclick = { move |_| cb.emit(r.id) }>
                            <td>{ r.match_number }</td>
                            <td>{ r.table_number }</td>
                            <td>{ r.status }</td>
                        </tr>
                    }
                })
            })
            .collect::<Html>();
        html! {
            <table class="table">
                <thead>
                    <tr>
                        <th>{ "Round" }</th>
                        <th>{ "Table" }</th>
                        <th>{ "Status" }</th>
                    </tr>
                </thead>
                <tbody> { list } </tbody>
            </table>
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct RoundSummary {
    pub id: RoundId,
    pub match_number: u64,
    pub table_number: u64,
    pub status: RoundStatus,
}

impl RoundSummary {
    pub fn new(rnd: &Round) -> Self {
        Self {
            id: rnd.id,
            match_number: rnd.match_number,
            table_number: rnd.table_number,
            status: rnd.status,
        }
    }
}
