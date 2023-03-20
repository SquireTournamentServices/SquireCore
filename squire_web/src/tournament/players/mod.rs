use yew::prelude::*;

use squire_sdk::tournaments::TournamentId;

pub mod filter;
pub mod input;
pub mod scroll;
pub mod selected;
pub use filter::*;
pub use input::*;
pub use scroll::*;
pub use selected::*;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct PlayersProps {
    pub id: TournamentId,
}

pub struct PlayersView {
    pub id: TournamentId,
}

impl Component for PlayersView {
    type Message = ();
    type Properties = PlayersProps;

    fn create(ctx: &Context<Self>) -> Self {
        PlayersView { id: ctx.props().id }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! { <h2> { "Players" } </h2> }
    }
}
