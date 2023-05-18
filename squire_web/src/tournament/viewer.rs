use futures::executor::block_on;
use gloo_net::http::Request;
use wasm_bindgen_futures::JsFuture;
use web_sys::RequestInit;
use yew::{html, Callback, Component, Context, Html, Properties};

use squire_sdk::{
    api::GET_TOURNAMENT_ROUTE,
    model::{admin::Admin, identifiers::AdminId},
    tournaments::{TournamentId, TournamentManager},
};

use crate::{
    tournament::{overview::*, players::*, rounds::*, settings::*, standings::*},
    utils::fetch_tournament,
    CLIENT, ON_UPDATE,
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
    TournamentImported,
    QueryReady(Option<(String, AdminId)>),
    SwitchModes(TournViewMode),
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct TournProps {
    pub id: TournamentId,
}

pub struct TournamentViewer {
    pub id: TournamentId,
    pub mode: TournViewMode,
    tourn_name: String,
    admin_id: AdminId,
}

impl TournamentViewer {
    fn get_header(&self, ctx: &Context<Self>) -> Html {
        let make_callback = |mode| {
            ctx.link()
                .callback(move |_| TournViewMessage::SwitchModes(mode))
        };
        let make_button = |name, mode| html! { <a class="py-2 px-1 text-center text-lg-start" onclick = { make_callback(mode) }>{name}</a> };
        html! {
            <div>
                <ul>
                    <h4 class="text-center text-lg-start">{ self.tourn_name.as_str() }</h4>
                    <hr/>
                    <li>{ make_button("Overview" , TournViewMode::Overview) }</li>
                    <li>{ make_button("Players"  , TournViewMode::Players) }</li>
                    <li>{ make_button("Rounds"   , TournViewMode::Rounds) }</li>
                    <li>{ make_button("Standings", TournViewMode::Standings) }</li>
                    <li>{ make_button("Settings" , TournViewMode::Settings) }</li>
                </ul>
            </div>
        }
    }

    fn get_control_plane(&self) -> Html {
        match self.mode {
            TournViewMode::Overview => {
                html! { <TournOverview id = { self.id }/> }
            }
            TournViewMode::Players => {
                html! { <PlayerView id = { self.id }/> }
            }
            TournViewMode::Rounds => {
                html! { <RoundsView id = { self.id } admin_id = { self.admin_id } /> }
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
            TournViewMessage::TournamentImported
        });
        Self {
            id,
            mode: TournViewMode::default(),
            tourn_name: String::new(),
            admin_id: AdminId::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TournViewMessage::SwitchModes(mode) => {
                let digest = mode != self.mode;
                self.mode = mode;
                digest
            }
            TournViewMessage::TournamentImported => {
                web_sys::console::log_1(&"Data ready!!".into());
                let id = self.id;
                ctx.link().send_future(async move {
                    let data = CLIENT
                        .get()
                        .unwrap()
                        .query_tourn(id, |t| {
                            let tourn = t.tourn();
                            (
                                tourn.name.clone(),
                                tourn.admins.keys().next().cloned().unwrap_or_default(),
                            )
                        })
                        .process()
                        .await;
                    TournViewMessage::QueryReady(data)
                });
                false
            }
            TournViewMessage::QueryReady(Some((name, admin_id))) => {
                web_sys::console::log_1(&format!("Tournament name ready and loaded!!").into());
                let digest = self.tourn_name != name;
                self.tourn_name = name;
                self.admin_id = admin_id;
                digest
            }
            TournViewMessage::QueryReady(None) => {
                let id = self.id;
                ctx.link().send_future(async move {
                    fetch_tournament(id).await;
                    TournViewMessage::TournamentImported
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="my-4 container-fluid">
                <div class="row tviewer">
                    <aside class="col-md-2 tveiwer_sidebar px-md-3">
                        { self.get_header(ctx) }
                    </aside>
                    <main class="col-md-10 conatiner">
                        { self.get_control_plane() }
                    </main>
                </div>
            </div>
        }
    }
}
