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

use crate::CLIENT;

use super::{roundresultticker, RoundResultTicker};

pub fn round_info_display(rnd: &Round) -> Html {
    html! {
        <>
            <p>{ format!("Round #{} at table #{}", rnd.match_number, rnd.table_number) }</p>
            <p>{ format!("Status : {}", rnd.status) }</p>
            <p>{ format!("Bye : {}", rnd.is_bye()) }</p>
        </>
    }
}

pub struct SelectedRound {
    pub(crate) id: RoundId,
    pub t_id: TournamentId,
    pub round_data_buffer: Option<Round>,
    draw_ticker: RoundResultTicker,
}

impl SelectedRound {
    pub fn new(id: RoundId, t_id: TournamentId) -> Self {
        Self {
            id,
            t_id,
            round_data_buffer: None,
            draw_ticker: RoundResultTicker {
                label: "Draws",
                result_type: RoundResult::Draw(0),
                stored_value: 0,
            },
        }
    }

    pub fn update(&mut self, id: RoundId) -> bool {
        let digest = self.id != id;
        self.id = id;
        if digest {
            self.round_data_buffer = CLIENT
                .get()
                .unwrap()
                .query_tourn(self.t_id, move |t| t.get_round(&id.into()).cloned().ok())
                .process()
                .flatten();
        }
        digest
    }

    pub fn view(&self, query: SelectedRoundQuery) -> Html {
        let returnhtml = self.round_data_buffer.as_ref()
            .map(|rnd| {
                // TODO: Remove unwrap here
                let dur_left = Duration::from_std(rnd.length + rnd.extension).unwrap() - (Utc::now() - rnd.timer);
                html! {
                    <>
                    <>{round_info_display(&rnd)}</>
                    <ul>
                    {
                        query.plyr_names.into_iter()
                            // Right now this code is duplicated, however once SelectedRound has more functionality it will be made significantly different. (It will have onclick functionality.)
                            .map(|(pid, name)| {
                                let player_wins = rnd.results.get(&pid).cloned().unwrap_or_default();
                                let player_confirm = rnd.confirmations.get(&pid).is_some();
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
                            .collect::<Html>()
                    }
                    </ul>
                    <p>
                    {
                        self.draw_ticker.view(rnd.draws)
                    }
                    </p>
                    <p>
                    { pretty_print_duration(dur_left) }
                    </p>
                    </>
                }
            })
            .unwrap_or_else(|| html!{
                <h4>{"Round not found"}</h4>
            });
        return html! {
            <div class="m-2">{returnhtml}</div>
        };
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

pub struct SelectedRoundQuery {
    plyr_names: HashMap<PlayerId, String>,
}

impl SelectedRoundQuery {
    pub fn new(plyrs: Vec<PlayerId>, tourn: &Tournament) -> Self {
        let plyr_names = plyrs
            .into_iter()
            .filter_map(|pid| {
                tourn
                    .get_player(&pid.into())
                    .map(|p| (pid, p.name.clone()))
                    .ok()
            })
            .collect();
        SelectedRoundQuery { plyr_names }
    }
}
