use js_sys::Math::round;
use squire_sdk::{
    model::{
        identifiers::RoundIdentifier,
        rounds::{Round, RoundId, RoundStatus},
    },
    tournaments::{Tournament, TournamentId},
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

use super::{input::RoundFilterReport, RoundsView, RoundsViewMessage, SelectedRoundMessage};

#[derive(Debug, PartialEq, Clone)]
pub enum RoundScrollMessage {
    ScrollQueryReady(Vec<RoundSummary>),
}

pub struct RoundScroll {
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
            .process()
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
            process: ctx.link().callback(SelectedRoundMessage::RoundSelected),
            rounds: Default::default(),
        }
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
        todo!()
        /*
        let RoundScrollQuery { sorted_rounds } = query;
        html! {
            <table class="table">
                <thead>
                    <tr>
                        <th>{ "Round" }</th>
                        <th>{ "Table" }</th>
                        <th>{ "Status" }</th>
                    </tr>
                </thead>
                <tbody>{
                    sorted_rounds.into_iter().map(|r| {
                        let cb = self.process.clone();
                        html! {
                            <tr onclick = { move |_| cb.emit(r.id) }>
                                <td>{ r.match_number }</td>
                                <td>{ r.table_number }</td>
                                <td>{ r.status }</td>
                            </tr>
                        }
                    }).collect::<Html>()
                }</tbody>
            </table>
        }
        */
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
    fn new(rnd: &Round) -> Self {
        Self {
            id: rnd.id,
            match_number: rnd.match_number,
            table_number: rnd.table_number,
            status: rnd.status,
        }
    }
}
