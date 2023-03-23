use yew::prelude::*;

use squire_sdk::{client::state::ClientState, tournaments::TournamentId};

mod general;
mod pairings;
mod scoring;

use general::*;
use pairings::*;
use scoring::*;

use crate::CLIENT;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct SettingsProps {
    pub id: TournamentId,
}

pub struct SettingsView {
    pub id: TournamentId,
    general: GeneralSettings,
    pairings: PairingsSettings,
    scoring: ScoringSettings,
}

impl Component for SettingsView {
    type Message = ();
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        SettingsView {
            id: ctx.props().id,
            general: Default::default(),
            pairings: Default::default(),
            scoring: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |tourn| {
                html! {
                    <div>
                        { self.general.view(tourn) }
                        { self.pairings.view(tourn) }
                        { self.scoring.view(tourn) }
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
