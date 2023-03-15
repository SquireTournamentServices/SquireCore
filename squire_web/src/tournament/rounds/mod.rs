use yew::prelude::*;

use squire_sdk::tournaments::TournamentId;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct RoundsProps {
    pub id: TournamentId,
}

pub struct RoundsView {
    pub id: TournamentId,
}

impl Component for RoundsView {
    type Message = ();
    type Properties = RoundsProps;

    fn create(ctx: &Context<Self>) -> Self {
        RoundsView { id: ctx.props().id }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{ <h2> { "Rounds" } </h2> }
    }
}
