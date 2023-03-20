use yew::prelude::*;

use squire_sdk::tournaments::TournamentId;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct SettingsProps {
    pub id: TournamentId,
}

pub struct SettingsView {
    pub id: TournamentId,
}

impl Component for SettingsView {
    type Message = ();
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        SettingsView { id: ctx.props().id }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! { <h2> { "Settings" } </h2> }
    }
}
