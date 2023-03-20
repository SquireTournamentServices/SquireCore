use yew::prelude::*;

use squire_sdk::{client::state::ClientState, tournaments::TournamentId};

use crate::CLIENT;

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
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |t| {
                html! {
                    <h1 align="center">{ format!("Welcome to {}", t.name) }</h1>
                }
            })
            .unwrap_or_default()
    }
}
