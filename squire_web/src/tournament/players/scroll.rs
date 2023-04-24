use squire_sdk::{
    model::{identifiers::RoundIdentifier, players::PlayerStatus, rounds::RoundStatus},
    players::{Player, PlayerId},
    tournaments::{Tournament, TournamentId},
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

use super::input::PlayerFilterReport;

pub struct PlayerScroll {
    pub process: Callback<PlayerId>,
    pub report: PlayerFilterReport,
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

    pub fn view(&self, query: PlayerScrollQuery) -> Html {
        let PlayerScrollQuery { sorted_players } = query;
        html! {
            <ul>
            {
                sorted_players.into_iter()
                    .map(|p| {
                        let cb = self.process.clone();
                        html! { <li><a class="py-1 vert" onclick = { move |_| cb.emit(p.id) }>{ p.name.as_str() }</a></li> }
                    })
                    .collect::<Html>()
            }
            </ul>
        }
    }
}

pub struct PlayerScrollQuery {
    sorted_players: Vec<PlayerSummary>,
}

impl PlayerScrollQuery {
    pub fn new(report: PlayerFilterReport, tourn: &Tournament) -> Self {
        let mut players: Vec<_> = tourn
            .player_reg
            .players
            .values()
            .filter_map(|p| report.matches(p).then(|| PlayerSummary::new(p)))
            .collect();
        players.sort_by_cached_key(|p| p.name.clone());
        players.sort_by_cached_key(|p| p.status);
        Self {
            sorted_players: players,
        }
    }
}

struct PlayerSummary {
    name: String,
    status: PlayerStatus,
    id: PlayerId,
}

impl PlayerSummary {
    fn new(plyr: &Player) -> Self {
        Self {
            name: plyr.name.clone(),
            status: plyr.status,
            id: plyr.id,
        }
    }
}
