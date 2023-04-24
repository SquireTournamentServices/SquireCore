use squire_sdk::{
    model::{
        identifiers::{PlayerIdentifier, TypeId},
        rounds::RoundId,
    },
    players::{Player, PlayerId, Round},
    tournaments::Tournament,
};
use yew::prelude::*;

use crate::tournament::rounds::{round_info_display, RoundSummary};

pub fn player_info_display(query: &SelectedPlayerQuery) -> Html {
    todo!()
    /*
     */
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
    pub id: Option<PlayerId>,
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

    fn subview_round(&self) -> Html {
        todo!()
        /*
        tourn
            .get_round(&rid.into())
            .map(|rnd| {
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
            .unwrap_or_else(|_| {
                html! {
                    <p>{ "Round not found." }</p>
                }
            })
        */
    }

    fn subview(&self) -> Html {
        match &self.spi {
            None => {
                html! { <h3>{" No info selected "}</h3> }
            }
            Some(SelectedPlayerInfo::Round(rid)) => self.subview_round(),
            Some(SelectedPlayerInfo::Deck(d_name)) => {
                html! { <p>{" Deck view hasn't been implemented :/ sorry."}</p> }
            }
        }
    }

    pub fn view(&self, query: SelectedPlayerQuery) -> Html {
        let SelectedPlayerQuery {
            gamer_tag,
            can_play,
            rounds,
            name,
        } = query;
        html! {
            <div class="m-2">
                <div class="row">
                    <div class="col">
                        <>{
                            html! {
                                <>
                                    <h4>{ name.as_str() }</h4>
                                    <p>{ format!("Gamertag : {}", gamer_tag.unwrap_or_default() ) }</p>
                                    <p>{ format!("Can play : {can_play}") }</p>
                                    <p>{ format!("Rounds : {}", rounds.len()) }</p>
                                </>
                            }
                        }</>
                        <ul>
                        {
                            rounds
                            .into_iter()
                            .map(|r| {
                                let cb = self.process.clone();
                                html! {<li class="sub_option" onclick={ move |_| cb.emit(SelectedPlayerInfo::Round(r.id)) }>{ format!("Match {} at table {}", r.match_number, r.table_number) }</li>}
                            })
                            .collect::<Html>()
                        }
                        </ul>
                    </div>
                    <div class="col">{ self.subview() }</div>
                </div>
            </div>
        }
    }
}

pub struct SelectedPlayerQuery {
    name: String,
    gamer_tag: Option<String>,
    can_play: bool,
    rounds: Vec<RoundSummary>,
}

impl SelectedPlayerQuery {
    pub fn new(pid: PlayerId, tourn: &Tournament) -> Self {
        todo!()
    }
}
