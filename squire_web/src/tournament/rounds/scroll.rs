use js_sys::Math::round;
use squire_sdk::{
    client::state::ClientState,
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
    report: RoundFilterReport,
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

    pub fn view(&self, tourn: &Tournament) -> Html {
        let unsorted_rounds = 
        tourn
        .round_reg
        .rounds
        .values()
        .filter(|r| self.report.matches(r));
        let mut rounds_vec = unsorted_rounds.collect::<Vec<_>>();
        rounds_vec.sort_by_cached_key(|r| r.match_number);
        rounds_vec.sort_by_cached_key(|r| r.status);
        let sorted_rounds = rounds_vec.into_iter();
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
                    sorted_rounds.map(|r| {
                        let id = r.id;
                        let cb = self.process.clone();
                        html! { 
                            <tr onclick = { move |_| cb.emit(id) }>
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
