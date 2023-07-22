use squire_sdk::model::{
    identifiers::{AdminId, TournamentId},
    operations::OpResult,
};
use tokio::sync::broadcast::Receiver as Subscriber;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDialogElement};
use yew::{html, Component, Context, Html, Properties};

use crate::{
    tournament::{overview::*, pairings::*, players::*, rounds::*, settings::*, standings::*},
    CLIENT,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TournViewMode {
    #[default]
    Overview,
    Players,
    Rounds,
    Pairings,
    Standings,
    Settings,
}

#[derive(Debug)]
pub enum TournViewMessage {
    TournamentImported(Option<Subscriber<bool>>),
    QueryReady(Option<(String, AdminId)>),
    SwitchModes(TournViewMode),
    TournamentUpdated(OpResult),
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct TournProps {
    pub id: TournamentId,
}

pub struct TournamentViewer {
    pub id: TournamentId,
    pub mode: TournViewMode,
    listener: Option<Subscriber<bool>>,
    tourn_name: String,
    admin_id: AdminId,
    error_message: String,
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
                    <li>{ make_button("Pairings" , TournViewMode::Pairings) }</li>
                    <li>{ make_button("Standings", TournViewMode::Standings) }</li>
                    <li>{ make_button("Settings" , TournViewMode::Settings) }</li>
                </ul>
            </div>
        }
    }

    fn get_control_plane(&self, ctx: &Context<Self>) -> Html {
        match self.mode {
            TournViewMode::Overview => {
                html! { <TournOverview id = { self.id }/> }
            }
            TournViewMode::Players => {
                let send_op_result = ctx.link().callback(TournViewMessage::TournamentUpdated);
                html! { <PlayerView id = { self.id } admin_id = { self.admin_id } send_op_result = { send_op_result } /> }
            }
            TournViewMode::Rounds => {
                let send_op_result = ctx.link().callback(TournViewMessage::TournamentUpdated);
                html! { <RoundsView id = { self.id } admin_id = { self.admin_id } send_op_result = { send_op_result } /> }
            }
            TournViewMode::Pairings => {
                let send_op_result = ctx.link().callback(TournViewMessage::TournamentUpdated);
                html!( <PairingsView id = {self.id } admin_id = { self.admin_id } send_op_result = { send_op_result } /> )
            }
            TournViewMode::Standings => {
                html! { <StandingsView id = { self.id }/> }
            }
            TournViewMode::Settings => {
                let send_op_result = ctx.link().callback(TournViewMessage::TournamentUpdated);
                html! { <SettingsView id = { self.id } admin_id = { self.admin_id } send_op_result = { send_op_result } /> }
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
            let res = CLIENT.get().unwrap().sub_to_tournament(id).await;
            TournViewMessage::TournamentImported(res)
        });
        Self {
            id,
            mode: TournViewMode::default(),
            tourn_name: String::new(),
            admin_id: AdminId::default(),
            error_message: "no message".to_owned(),
            listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TournViewMessage::SwitchModes(mode) => {
                let digest = mode != self.mode;
                self.mode = mode;
                digest
            }
            TournViewMessage::TournamentImported(listener) => {
                let id = self.id;
                self.listener = listener;
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
                let digest = self.tourn_name != name;
                self.tourn_name = name;
                self.admin_id = admin_id;
                digest
            }
            TournViewMessage::QueryReady(None) => {
                let id = self.id;
                ctx.link().send_future(async move {
                    let res = CLIENT.get().unwrap().sub_to_tournament(id).await;
                    TournViewMessage::TournamentImported(res)
                });
                false
            }
            TournViewMessage::TournamentUpdated(opr) => {
                let Err(err) = opr else { return false };
                let element: HtmlDialogElement = window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("errormessage"))
                    .and_then(|e| e.dyn_into::<HtmlDialogElement>().ok())
                    .unwrap();
                self.error_message = err.to_string();
                let _ = element.show_modal();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <dialog id="errormessage">
                <p>{self.error_message.clone()}</p>
                <form method="dialog">
                <button>{"OK"}</button>
                </form>
            </dialog>

            <div class="my-4 container-fluid">
                <div class="row tviewer">
                    <aside class="col-md-2 tveiwer_sidebar px-md-3">
                        { self.get_header(ctx) }
                    </aside>
                    <main class="col-md-10 conatiner">
                        { self.get_control_plane(ctx) }
                    </main>
                </div>
            </div>
            </>
        }
    }
}
