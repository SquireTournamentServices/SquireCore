use squire_sdk::{
    players::{Player, PlayerId, Round},
    tournaments::Tournament, model::{identifiers::{PlayerIdentifier, TypeId}, rounds::RoundId},
};
use yew::prelude::*;

use crate::tournament::rounds::round_info_display;

pub fn player_info_display(tourn: &Tournament, plyr: &Player) -> Html {
    let gamertag = { |plyr: &Player|
        plyr.game_name.
        clone()
        .unwrap_or_else(|| "None".to_string())
    };
    let round_number = { |plyr: &Player|
        tourn.
        get_player_rounds(&plyr.id.into()).
        unwrap_or_default().
        len()
    };
    html! {
        <>
            <h4>{ plyr.name.as_str() }</h4>
            <p>{ format!("Gamertag : {}", gamertag(plyr) ) }</p>
            <p>{ format!("Can play : {}", plyr.can_play()) }</p>
            <p>{ format!("Rounds : {}", round_number(plyr) ) }</p>
        </>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectedPlayerInfo {
    Round(RoundId),
    Deck(String),
}
pub enum SelectedPlayerMessage {
    PlayerSelected(Option<PlayerId>),
    InfoSelected(Option<SelectedPlayerInfo>),
}

pub struct SelectedPlayer {
    pub process: Callback<SelectedPlayerInfo>,
    id: Option<PlayerId>,
    spi: Option<SelectedPlayerInfo>,
}

impl SelectedPlayer {
    pub fn new(process: Callback<SelectedPlayerInfo>) -> Self {
        Self { 
            process,
            id: None,
            spi: None,
        }
    }

    pub fn update(&mut self, msg: SelectedPlayerMessage) -> bool {
        match msg {
            SelectedPlayerMessage::PlayerSelected(p_id) => {
                let digest = self.id != p_id;
                self.spi = None;
                self.id = p_id;
                digest
            }
            SelectedPlayerMessage::InfoSelected(spi) => {
                let digest = self.spi != spi;
                self.spi = spi;
                digest
            }
        }
    }


    fn subview_round(&self, tourn: &Tournament, rid: RoundId) -> Html {
        tourn.get_round(&rid.into()).map(|rnd|{
            html! {
                html! {
                    <>
                    <>{round_info_display(rnd)}</>
                    <ul>
                    {
                        rnd.players.clone().into_iter()
                            .map(|pid| {
                                let player_in_round = { ||
                                    tourn
                                    .get_player(&pid.into())
                                    .map(|p| p.name.as_str())
                                    .unwrap_or_else( |_| "Player not found")
                                };
                                html! { <li>{ format!( "{}", player_in_round() ) }</li> }
                            })
                            .collect::<Html>()
                    }
                    </ul>
                    </>
                }
            }
        })
        .unwrap_or_else(|_| html!{
            <p>{ "Round not found." }</p>
        })
    }
    fn subview(&self, tourn: &Tournament) -> Html {
        let spi = self.spi.clone();
        match spi {
            None => {
                html!{ <h3>{" No info selected "}</h3> }
            }
            Some(SelectedPlayerInfo::Round(rid)) => {
                self.subview_round(tourn, rid)
            }
            Some(SelectedPlayerInfo::Deck(d_name)) => {
                html!{ <p>{" Deck view hasn't been implemented :/ sorry."}</p> }
            }
        }
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
                                    <>{player_info_display(tourn, plyr)}</>
                                    <ul>
                                    {
                                        tourn.get_player_rounds(&id.into())
                                        .unwrap_or_default()
                                        .into_iter()
                                        .map(|r| {
                                            let rid = r.id;
                                            let cb = self.process.clone();
                                            html! {<li class="sub_option" onclick={ move |_| cb.emit(SelectedPlayerInfo::Round(rid)) }>{ format!("Match {} at table {}", r.match_number, r.table_number) }</li>}
                                        })
                                        .collect::<Html>()
                                    }
                                    </ul>
                                </div>
                                <div class="col">{ self.subview(tourn) }</div>
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
        html!{
            <div class="m-2">{returnhtml}</div>
        }
    }
}
