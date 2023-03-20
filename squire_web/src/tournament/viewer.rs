use futures::executor::block_on;
use gloo_net::http::Request;
use wasm_bindgen_futures::JsFuture;
use web_sys::RequestInit;
use yew::{html, Callback, Component, Context, Html, Properties};

use squire_sdk::{
    api::GET_TOURNAMENT_ROUTE,
    client::state::ClientState,
    tournaments::{TournamentId, TournamentManager},
};

use crate::{
    client,
    tournament::{overview::*, players::*, rounds::*, settings::*, standings::*},
    CLIENT, utils::fetch_tournament,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TournViewMode {
    #[default]
    Overview,
    Players,
    Rounds,
    Standings,
    Settings,
}

#[derive(Debug)]
pub enum TournViewMessage {
    DataReady,
    SwitchModes(TournViewMode),
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct TournProps {
    pub id: TournamentId,
}

pub struct TournamentViewer {
    pub id: TournamentId,
    pub mode: TournViewMode,
}

impl TournamentViewer {
    fn get_header(&self, ctx: &Context<Self>) -> Html {
        let make_callback = |mode| {
            ctx.link().callback(move |_| {
                web_sys::console::log_1(&format!("{mode:?}").into());
                TournViewMessage::SwitchModes(mode)
            })
        };
        let make_button = |name, mode| html! { <a align="center" class="vert" onclick = { make_callback(mode) }>{name}</a> };
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |t| {
                let tourn = t.tourn();
                html! {
                    <div>
                        <ul>
                            <li>{ make_button("Overview" , TournViewMode::Overview) }</li>
                            <li>{ make_button("Players"  , TournViewMode::Players) }</li>
                            <li>{ make_button("Rounds"   , TournViewMode::Rounds) }</li>
                            <li>{ make_button("Standings", TournViewMode::Standings) }</li>
                            <li>{ make_button("Settings" , TournViewMode::Settings) }</li>
                        </ul>
                    </div>
                }
            })
            .unwrap_or_default()
    }

    fn get_control_plane(&self) -> Html {
        match self.mode {
            TournViewMode::Overview => {
                html! { <TournOverview id = { self.id }/> }
            }
            TournViewMode::Players => {
                html! { <PlayerFilter id = { self.id }/> }
            }
            TournViewMode::Rounds => {
                html! { <RoundsView id = { self.id }/> }
            }
            TournViewMode::Standings => {
                html! { <StandingsView id = { self.id }/> }
            }
            TournViewMode::Settings => {
                html! { <SettingsView id = { self.id }/> }
            }
        }
    }
}

impl Component for TournamentViewer {
    type Message = TournViewMessage;
    type Properties = TournProps;

    fn create(ctx: &Context<Self>) -> Self {
        let TournProps { id } = ctx.props();
        let id = *id;
        ctx.link().send_future(async move {
            fetch_tournament(id).await;
            TournViewMessage::DataReady
        });
        Self {
            id,
            mode: TournViewMode::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TournViewMessage::SwitchModes(mode) => {
                if mode != self.mode {
                    self.mode = mode;
                    true
                } else {
                    false
                }
            }
            TournViewMessage::DataReady => {
                let client = CLIENT.get().unwrap();
                if client.state.query_tournament(&self.id, |_| ()).is_none() {
                    let id = self.id;
                    ctx.link().send_future(async move {
                        fetch_tournament(id).await;
                        TournViewMessage::DataReady
                    });
                    false
                } else {
                    true
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let client = CLIENT.get().unwrap();
        client
            .state
            .query_tournament(&self.id, |t| {
                let tourn = t.tourn();
                html! {
                    <div>
                        { self.get_header(ctx) }
                        { self.get_control_plane() }
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
