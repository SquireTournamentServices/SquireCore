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
    ctx.link()
        .send_future(async move { recv.recv().await.map(|_| msg).unwrap() })
}
