use squire_sdk::{
    client::SquireClient,
    model::{identifiers::AdminId, operations::OpResult, tournament::TournamentId},
    sync::TournamentManager,
};
// use squire_sdk::tournaments::TournamentManager;
// use wasm_bindgen_futures::spawn_local;
use yew::{Callback, Component, Context, Properties};

use crate::CLIENT;

pub struct TournViewerComponentWrapper<T> {
    state: WrapperState,
    comp: T,
}
pub struct WrapperState {
    pub a_id: AdminId,
    pub t_id: TournamentId,
    pub send_op_result: Callback<OpResult>,
    pub client: &'static SquireClient,
}
pub enum InteractionResponse<T>
where
    T: TournViewerComponent,
{
    Redraw(bool),
    Update,
    FetchData(Box<dyn 'static + Send + FnOnce(&TournamentManager) -> T::QueryMessage>),
}
pub enum WrapperMessage<T>
where
    T: TournViewerComponent,
{
    // User interaction with the component when doing something like clicking
    Interaction(T::InteractionMessage),
    // Message to query all of the information for the component
    ReQuery,
    // Message to query individual bits of information
    QueryData(T::QueryMessage),
    // Message from the server telling the component there has been an update
    RemoteUpdate(TournamentId),
}
#[derive(PartialEq, Properties)]
pub struct WrapperProps<P>
where
    P: PartialEq,
{
    pub t_id: TournamentId,
    pub a_id: AdminId,
    pub send_op_result: Callback<OpResult>,
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
            a_id: ctx.props().a_id,
            t_id: ctx.props().t_id,
            send_op_result: ctx.props().send_op_result,
            client: CLIENT.get().unwrap(),
        };
        let mut comp = T::v_create(ctx, &state);
        let q_func = comp.query(ctx, &state);
        let to_return = TournViewerComponentWrapper { state, comp };
        to_return.query_tourn(ctx, q_func);
        to_return
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WrapperMessage::Interaction(msg) => {
                match self.comp.interaction(ctx, msg, &self.state) {
                    InteractionResponse::Redraw(value) => value,
                    InteractionResponse::Update => todo!(), // <-- where tourn ops happen
                    InteractionResponse::FetchData(q_func) => self.query_tourn(ctx, q_func),
                }
            }
            WrapperMessage::ReQuery => {
                self.comp.query(ctx, &self.state);
                false
            }
            WrapperMessage::QueryData(data) => self.comp.load_queried_data(data, &self.state),
            WrapperMessage::RemoteUpdate(t_id) => {
                if self.state.t_id == t_id {
                    self.comp.query(ctx, &self.state);
                }
                false
            }
        }
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        T::v_view(&self.comp, _ctx.into(), &self.state)
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
}

pub trait TournViewerComponent: Sized + 'static {
    type InteractionMessage;
    type QueryMessage: 'static + Send;
    type Properties: Properties;

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self;

    fn load_queried_data(&mut self, _msg: Self::QueryMessage, state: &WrapperState) -> bool {
        false
    }

    fn interaction(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        msg: Self::InteractionMessage,
        state: &WrapperState,
    ) -> InteractionResponse<Self> {
        false.into()
    }

    fn query(
        &mut self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        state: &WrapperState,
    ) -> Box<dyn 'static + Send + FnOnce(&TournamentManager) -> Self::QueryMessage>;

    fn v_view(
        &self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        state: &WrapperState,
    ) -> yew::Html;
}

impl<T: TournViewerComponent> From<bool> for InteractionResponse<T> {
    fn from(value: bool) -> Self {
        InteractionResponse::Redraw(value)
    }
}

// yew::Context<TournViewerComponentWrapper<T>> -> yew::Context<T>
