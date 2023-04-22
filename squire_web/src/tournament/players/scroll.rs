use squire_sdk::{
    model::{identifiers::RoundIdentifier, rounds::RoundStatus},
    players::PlayerId,
    tournaments::{Tournament, TournamentId},
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

use super::input::PlayerFilterReport;

pub struct PlayerScroll {
    pub process: Callback<PlayerId>,
    report: PlayerFilterReport,
}

impl PlayerScroll {
    pub fn new(process: Callback<PlayerId>) -> Self {
        Self {
            process,
            report: Default::default(),
        }
    }

    pub fn update(&mut self, report: PlayerFilterReport) -> bool {
        let digest = self.report != report;
        self.report = report;
        digest
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        let unsorted_players = tourn
            .player_reg
            .players
            .values()
            .filter(|p| self.report.matches(p));
        let mut players_vec = unsorted_players.collect::<Vec<_>>();
        players_vec.sort_by_cached_key(|p| p.name.clone());
        players_vec.sort_by_cached_key(|p| p.status);
        html! {
            <ul>
            {
                players_vec.into_iter()
                    .map(|p| {
                        let id = p.id;
                        let cb = self.process.clone();
                        html! { <li><a class="py-1 vert" onclick = { move |_| cb.emit(id) }>{ p.name.as_str() }</a></li> }
                    })
                    .collect::<Html>()
            }
            </ul>
        }
    }
}
