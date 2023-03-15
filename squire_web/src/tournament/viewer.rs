use yew::{html, Callback, Component, Context, Html, Properties};

use squire_sdk::{client::state::ClientState, tournaments::TournamentId};

use crate::{
    client,
    tournament::{overview::*, players::*, rounds::*, settings::*, standings::*},
    CLIENT,
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
                        <h1 align="center">{ format!("Welcome to {}", tourn.name) }</h1>
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
                html! { <PlayersView id = { self.id }/> }
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
        Self {
            id: *id,
            mode: TournViewMode::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TournViewMessage::SwitchModes(mode) => {
                if mode != self.mode {
                    self.mode = mode;
                    true
                } else {
                    false
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
