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
pub mod viewer_component;

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
