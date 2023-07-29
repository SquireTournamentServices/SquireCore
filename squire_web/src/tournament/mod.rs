use squire_sdk::tournaments::TournamentId;
// use squire_sdk::tournaments::TournamentManager;
// use wasm_bindgen_futures::spawn_local;
use yew::{Component, Context};

use crate::ON_UPDATE;

pub mod creator;
pub mod model;
pub mod overview;
pub mod pairings;
pub mod players;
pub mod rounds;
pub mod settings;
pub mod standings;
pub mod viewer;

pub fn spawn_update_listener<V, M>(ctx: &Context<V>, msg: M)
where
    V: Component<Message = M>,
    M: 'static,
{
    let recv = ON_UPDATE.get().unwrap().clone();
    ctx.link().send_future(async move {
        let to_return = recv.recv().await.map(|_| msg).unwrap();
        to_return
    })
}


/*
pub struct TournViewerComponentWrapper<T> {
    comp: T,
}

enum WrapperMessage<T>
where T: TournViewerComponent
{
    Interaction(T::InteractionMessage),
    ReQuery,
    QueryData(T::QueryMessage),
    RemoteUpdate(TournamentId),
}

impl<T> Component for TournViewerComponentWrapper<T>
where T: TournViewerComponent + 'static
{
    type Message = WrapperMessage<T>;
    type Properties;

    fn create(ctx: &yew::Context<Self>) -> Self {
        todo!()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WrapperMessage::Interaction(msg) => self.comp.interaction(msg),
            WrapperMessage::ReQuery => self.comp.query(ctx),
            WrapperMessage::QueryData(data) => self.comp.load_queried_data(data),
            WrapperMessage::RemoteUpdate(t_id) if self.t_id == t_id => self.comp.query(ctx),
            _ => false,
        }
        todo!()
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        todo!()
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

    fn prepare_state(&self) -> Option<String> {
        None
    }

    fn destroy(&mut self, ctx: &Context<Self>) {}
}
pub trait TournViewerComponent {
    type InteractionMessage;
    type QueryMessage;

    fn load_queried_data(&mut self, msg: Self::QueryMessage) -> bool;
    
    fn interaction(&mut self, msg: Self::InteractionMessage) -> bool;

    fn query(&mut self, ctx: &Context<TournViewerComponentWrapper<Self>>);
}
*/