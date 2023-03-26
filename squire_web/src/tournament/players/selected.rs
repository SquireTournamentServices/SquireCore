use squire_sdk::{
    players::{Player, PlayerId, Round},
    tournaments::Tournament, model::{identifiers::{PlayerIdentifier, TypeId}, rounds::RoundId},
};
use yew::prelude::*;

pub struct SelectedPlayer {
    pub process: Callback<RoundId>,
    id: Option<PlayerId>,
    round_id: Option<RoundId>,
}

impl SelectedPlayer {
    pub fn new(process: Callback<RoundId>) -> Self {
        Self { 
            process,
            id: None,
            round_id: None,
        }
    }

    pub fn update(&mut self, id: Option<PlayerId>) -> bool {
        let digest = self.id != id;
        self.id = id;
        digest
    }

    pub fn update_round(&mut self, rid: Option<RoundId>) -> bool {
        let digest = self.round_id != rid;
        self.round_id = rid;
        digest
    }

    fn subview_match(&self, tourn: &Tournament) -> Html {
        self
            .round_id
            .map(|round_id| {
                tourn
                    .get_round(&round_id.into())
                    .map(|rnd| {
                        html! {
                            <>
                            <p>{ format!("Round #{} at table #{}", rnd.match_number, rnd.table_number) }</p>
                            <p>{ format!("Active : {}", rnd.is_active()) }</p>
                            <p>{ format!("Players : {}", rnd.players.len() ) }</p>
                            <ul>
                            {
                                rnd.players.clone().into_iter()
                                    .map(|pid| {
                                        html! { <li>{ format!( "{}", tourn.get_player(&pid.into()).map(|p| p.name.as_str()).unwrap_or_else(|_| "Player not found") ) }</li>}
                                    })
                                    .collect::<Html>()
                            }
                            </ul>
                            </>
                        }
                    })
                    .unwrap_or_else(|_| html!{
                        <p>{"Match not found"}</p>
                    })
            })
            .unwrap_or_else(|| html!{
                <p>{"No match selected"}</p>
            })
        }

    pub fn view(&self, tourn: &Tournament) -> Html {
        let returnhtml = self
            .id
            .map(|id| {
                tourn
                    .get_player(&id.into())
                    .map(|plyr| {
                        html! {
                            <div class="row">
                                <div class="col">
                                    <h4>{ plyr.name.as_str() }</h4>
                                    <p>{ format!("Gamertag : {}", plyr.game_name.clone().unwrap_or_else(|| "None".to_string())) }</p>
                                    <p>{ format!("Can play : {}", plyr.can_play()) }</p>
                                    <p>{ format!("Rounds : {}", tourn.get_player_rounds(&id.into()).unwrap_or_default().len() ) }</p>
                                    <ul>
                                    {
                                        tourn.get_player_rounds(&id.into()).unwrap_or_default().into_iter()
                                            .map(|r| {
                                                let rid = r.id;
                                                let cb = self.process.clone();
                                                html! {<li class="sub_option" onclick={ move |_| cb.emit(rid) }>{ format!("Match {} at table {}", r.match_number, r.table_number) }</li>}
                                            })
                                            .collect::<Html>()
                                    }
                                    </ul>
                                </div>
                                <div class="col">{ self.subview_match(tourn) }</div>
                            </div>
                        }
                    })
                    .unwrap_or_else(|_| html!{
                        <h4>{"Player not found"}</h4>
                    })
            })
            .unwrap_or_else(|| html!{
                <h4>{"No player selected"}</h4>
            });
        return html!{
            <div class="m-2">{returnhtml}</div>
        };
    }
}
