#![allow(non_camel_case_types)]

use derive_more::From;
use squire_sdk::{
    client::SquireClient,
    model::{
        admin::TournOfficialId,
        identifiers::SquireAccountId,
        operations::{AdminOp, JudgeOp, OpResult, TournOp},
        tournament::TournamentId,
    },
    sync::TournamentManager,
};
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDialogElement};
use yew::{html, Component, Context, Properties};

use crate::{utils::console_log, CLIENT, ON_UPDATE};

pub mod creator;
pub mod model;
pub mod overview;
pub mod pairings;
pub mod players;
pub mod rounds;
pub mod settings;
pub mod standings;
pub mod viewer;

pub struct TournViewerComponentWrapper<T> {
    state: WrapperState,
    error_message: String,
    comp: T,
}

pub enum Op {
    Admin(AdminOp),
    Judge(JudgeOp),
}

pub struct WrapperState {
    pub t_id: TournamentId,
    pub client: &'static SquireClient,
}

impl WrapperState {
    pub fn get_user_id(&self) -> Option<SquireAccountId> {
        self.client.get_user().map(|acc| acc.id)
    }
    pub fn op_response<T: TournViewerComponent>(
        &self,
        operations: Vec<Op>,
    ) -> InteractionResponse<T> {
        self.get_user_id()
            .map(|user_id| {
                let mut ops: Vec<TournOp> = Vec::new();
                ops.extend(operations.into_iter().map(|op| match op {
                    Op::Admin(a_op) => TournOp::AdminOp(user_id.convert(), a_op),
                    Op::Judge(j_op) => {
                        TournOp::JudgeOp(TournOfficialId::Judge(user_id.convert()), j_op)
                    }
                }));
                InteractionResponse::Update(ops)
            })
            .unwrap_or_default()
    }
}

#[derive(From)]
pub enum InteractionResponse<T>
where
    T: TournViewerComponent,
{
    Redraw(bool),
    Update(Vec<TournOp>), // <- We probably want to pass in a client Update type instead
    #[from(ignore)]
    FetchData(TournQuery<T::QueryMessage>),
    NoOp,
}

#[derive(From)]
pub enum WrapperMessage<T>
where
    T: TournViewerComponent,
{
    /// User interaction with the component when doing something like clicking
    #[from(ignore)]
    Interaction(T::InteractionMessage),
    /// Message to query all of the information for the component
    ReQuery,
    /// Message to query individual bits of information
    #[from(ignore)]
    QueryData(T::QueryMessage),
    /// Message from the server telling the component there has been an update
    RemoteUpdate(TournamentId),
    /// Will display an error message if the operation result is an error
    ReceiveOpResult(OpResult),
}

#[derive(PartialEq, Properties)]
pub struct WrapperProps<P>
where
    P: PartialEq,
{
    pub t_id: TournamentId,
    pub props: P,
}

impl<T> Component for TournViewerComponentWrapper<T>
where
    T: TournViewerComponent + 'static,
{
    type Message = WrapperMessage<T>;
    type Properties = WrapperProps<<T as TournViewerComponent>::Properties>;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let state = WrapperState {
            t_id: ctx.props().t_id,
            client: CLIENT.get().unwrap(),
        };
        let mut comp = T::v_create(ctx, &state);
        let q_func = comp.query(ctx, &state);
        let to_return = TournViewerComponentWrapper {
            state,
            error_message: "".to_owned(),
            comp,
        };
        to_return.spawn_update_listener(ctx);
        to_return.query_tourn(ctx, q_func);
        to_return
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WrapperMessage::Interaction(msg) => {
                match self.comp.interaction(ctx, msg, &self.state) {
                    InteractionResponse::Redraw(value) => value,
                    InteractionResponse::Update(ops) => {
                        let handle = CLIENT.get().unwrap().bulk_update(self.state.t_id, ops);
                        let is_success = ctx.link().callback(move |_| WrapperMessage::ReQuery);
                        ctx.link().send_future(async move {
                            let op_result = handle.await.unwrap();
                            if op_result.is_ok() {
                                is_success.emit(())
                            };
                            op_result
                        });
                        false
                    }
                    InteractionResponse::FetchData(q_func) => {
                        self.query_tourn(ctx, q_func);
                        true
                    }
                    InteractionResponse::NoOp => false,
                }
            }
            WrapperMessage::ReQuery => {
                let q_func = self.comp.query(ctx, &self.state);
                self.query_tourn(ctx, q_func);
                false
            }
            WrapperMessage::QueryData(data) => self.comp.load_queried_data(data, &self.state),
            WrapperMessage::RemoteUpdate(t_id) => {
                if self.state.t_id == t_id {
                    let _ = self.comp.query(ctx, &self.state);
                }
                false
            }
            WrapperMessage::ReceiveOpResult(opr) => {
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

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        html!(
            <>
                <>{ T::v_view(&self.comp, _ctx.into(), &self.state)} </>
                <>
                    <dialog id="errormessage">
                    <p>{self.error_message.clone()}</p>
                    <form method="dialog">
                    <button>{"OK"}</button>
                    </form>
                    </dialog>
                </>
            </>
        )
    }
}
impl<T> TournViewerComponentWrapper<T>
where
    T: TournViewerComponent + 'static,
{
    fn query_tourn<F>(&self, ctx: &yew::Context<Self>, q_func: F)
    where
        F: 'static + Send + FnOnce(&TournamentManager) -> T::QueryMessage,
    {
        let handle = self.state.client.query_tourn(self.state.t_id, q_func);
        ctx.link()
            .send_future(async move { WrapperMessage::QueryData(handle.await.unwrap()) });
    }

    fn spawn_update_listener(&self, ctx: &Context<Self>) {
        console_log("Spawning update listener");
        let recv = ON_UPDATE.get().unwrap().clone();
        ctx.link().send_future(async move {
            recv.recv().await.map(WrapperMessage::RemoteUpdate).unwrap()
        })
    }
}

pub type TournQuery<T> = Box<dyn 'static + Send + FnOnce(&TournamentManager) -> T>;

pub trait TournViewerComponent: Sized + 'static {
    type InteractionMessage;
    type QueryMessage: 'static + Send;
    type Properties: Properties;

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self;

    #[allow(unused_variables)]
    fn load_queried_data(&mut self, msg: Self::QueryMessage, state: &WrapperState) -> bool {
        false
    }

    #[allow(unused_variables)]
    fn interaction(
        &mut self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        msg: Self::InteractionMessage,
        state: &WrapperState,
    ) -> InteractionResponse<Self> {
        false.into()
    }

    fn query(
        &mut self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        state: &WrapperState,
    ) -> TournQuery<Self::QueryMessage>;

    fn v_view(
        &self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        state: &WrapperState,
    ) -> yew::Html;
}

impl<T: TournViewerComponent> Default for InteractionResponse<T> {
    fn default() -> Self {
        Self::NoOp
    }
}
