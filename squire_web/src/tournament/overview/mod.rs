use yew::prelude::*;

use squire_sdk::tournaments::TournamentId;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct OverviewProps {
    pub id: TournamentId,
}

pub struct TournOverview {
    pub id: TournamentId,
}

impl Component for TournOverview {
    type Message = ();
    type Properties = OverviewProps;

    fn create(ctx: &Context<Self>) -> Self {
        TournOverview { id: ctx.props().id }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!{ <h2> { "Overview" } </h2> }
    }
}
