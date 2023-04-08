use chrono::{Duration, Utc};
use squire_sdk::{model::{
    rounds::{Round, RoundId, RoundStatus},
    tournament::Tournament,
}, tournaments::TournamentId, client::state::ClientState};
use yew::prelude::*;

use crate::CLIENT;

pub fn round_info_display(rnd: &Round) -> Html {
    let round_status = { ||
        match rnd.status {
            RoundStatus::Open => "Open",
            RoundStatus::Certified => "Certified",
            RoundStatus::Dead => "Dead"

        }
    };

    html! {
        <>
            <p>{ format!("Round #{} at table #{}", rnd.match_number, rnd.table_number) }</p>
            <p>{ format!("Status : {}", round_status()) }</p>
            <p>{ format!("Bye : {}", rnd.is_bye()) }</p>
        </>
    }
}

pub struct SelectedRound {
    pub(crate) id: RoundId,
    pub t_id: TournamentId,
    round_data_buffer: Option<Round>,
}

impl SelectedRound {
    pub fn new(id: RoundId, t_id: TournamentId) -> Self {
        Self {
            id,
            t_id,
            round_data_buffer : None,
        }
    }

    pub fn update(&mut self, id: RoundId) -> bool {
        let digest = self.id != id;
        self.id = id;
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.t_id, |t| {
                self.round_data_buffer = t.get_round(&self.id.into())
                    .map(|r| Some(r.clone()))
                    .unwrap_or(None)
            });
        digest
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        let returnhtml = self.round_data_buffer.as_ref()
            .map(|rnd| {
                // TODO: Remove unwrap here
                let dur_left = Duration::from_std(rnd.length + rnd.extension).unwrap() - (Utc::now() - rnd.timer);
                html! {
                    <>
                    <>{round_info_display(&rnd)}</>
                    <ul>
                    {
                        rnd.players.clone().into_iter()
                            // Right now this code is duplicated, however once SelectedRound has more functionality it will be made significantly different. (It will have onclick functionality.)
                            .map(|pid| {
                                let player_in_round = { ||
                                    tourn
                                    .get_player(&pid.into())
                                    .map(|p| p.name.as_str())
                                    .unwrap_or_else( |_| "Player not found")
                                };
                                let player_wins = rnd.results.get(&pid.into()).unwrap_or(&0);
                                let player_confirm = rnd.confirmations.get(&pid.into()).is_some();
                                html! { 
                                    <li>
                                    <div>
                                    { format!( "{}", player_in_round()) }
                                    </div>
                                    <div>
                                    { format!( "wins : {}, confirmed : {}", player_wins, player_confirm ) }
                                    </div>
                                    </li>
                                }
                            })
                            .collect::<Html>()
                    }
                    </ul>
                    <p>
                    { format!("Draws : {}", rnd.draws) }
                    </p>
                    <p>
                    { 
                        pretty_print_duration(dur_left)
                    }
                    </p>
                    </>
                }
            })
            .unwrap_or_else(|| html!{
                <h4>{"Round not found"}</h4>
            });
        return html!{
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
        format!("Over time: {}:{}:{}", hours.abs(), mins.abs() % 60, secs.abs() % 60)
    }
}
