use squire_sdk::model::{
    rounds::{Round, RoundId},
    tournament::Tournament,
};
use yew::prelude::*;

pub fn round_info_display(rnd: &Round) -> Html {
    html! {
        <>
            <p>{ format!("Round #{} at table #{}", rnd.match_number, rnd.table_number) }</p>
            <p>{ format!("Active : {}", rnd.is_active()) }</p>
            <p>{ format!("Players : {}", rnd.players.len() ) }</p>
            <p>{ format!("Bye : {}", rnd.is_bye()) }</p>
        </>
    }
}

pub struct SelectedRound {
    id: Option<RoundId>,
}

impl SelectedRound {
    pub fn new() -> Self {
        Self { id: None }
    }

    pub fn update(&mut self, id: Option<RoundId>) -> bool {
        let digest = self.id != id;
        self.id = id;
        digest
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        let returnhtml = self
            .id
            .map(|id| {
                tourn
                    .get_round(&id.into())
                    .map(|rnd| {
                        html! {
                            <>
                            <>{round_info_display(rnd)}</>
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
                                        html! { <li>{ format!( "{}", player_in_round() ) }</li>}
                                    })
                                    .collect::<Html>()
                            }
                            </ul>
                            </>
                        }
                    })
                    .unwrap_or_else(|_| html!{
                        <h4>{"Round not found"}</h4>
                    })
            })
            .unwrap_or_else(|| html!{
                <h4>{"No round selected"}</h4>
            });
        return html!{
            <div class="m-2">{returnhtml}</div>
        };
    }
}
