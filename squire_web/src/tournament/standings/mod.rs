use yew::prelude::*;

use squire_sdk::tournaments::TournamentId;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct StandingsProps {
    pub id: TournamentId,
}

pub struct StandingsView {
    pub id: TournamentId,
}

impl Component for StandingsView {
    type Message = ();
    type Properties = StandingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        StandingsView { id: ctx.props().id }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! { <h2> { "Standings" } </h2> }
    }
}
