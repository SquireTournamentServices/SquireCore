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

use super::input::RoundFilterReport;

pub struct RoundScroll {
    pub process: Callback<RoundId>,
    pub report: RoundFilterReport,
}

impl RoundScroll {
    pub fn new(process: Callback<RoundId>) -> Self {
        Self {
            process,
            report: Default::default(),
        }
    }

    pub fn update(&mut self, report: RoundFilterReport) -> bool {
        let digest = self.report != report;
        self.report = report;
        digest
    }

    pub fn view(&self, query: RoundScrollQuery) -> Html {
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
    }
}

pub struct RoundSummary {
    pub id: RoundId,
    pub match_number: u64,
    pub table_number: u64,
    pub status: RoundStatus,
}

pub struct RoundScrollQuery {
    sorted_rounds: Vec<RoundSummary>,
}

impl RoundScrollQuery {
    pub fn new(report: RoundFilterReport, tourn: &Tournament) -> Self {
        let unsorted_rounds = tourn
            .round_reg
            .rounds
            .values()
            .filter_map(|r| report.matches(r).then(|| RoundSummary::new(r)));
        let mut sorted_rounds: Vec<_> = unsorted_rounds.collect();
        sorted_rounds.sort_by_cached_key(|r| r.match_number);
        sorted_rounds.sort_by_cached_key(|r| r.status);
        RoundScrollQuery { sorted_rounds }
    }
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
