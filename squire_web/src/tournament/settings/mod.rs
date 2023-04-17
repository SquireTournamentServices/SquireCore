use std::collections::HashSet;

use yew::prelude::*;

use squire_sdk::{
    accounts::TournamentSettingsTree, client::state::ClientState,
    model::settings::TournamentSetting, tournaments::TournamentId,
};

mod general;
mod pairings;
mod panel;
mod scoring;

use general::*;
use pairings::*;
use scoring::*;

use crate::CLIENT;

#[derive(Debug, PartialEq, Eq)]
pub enum SettingsMessage {
    Setting(TournamentSetting),
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct SettingsProps {
    pub id: TournamentId,
}

pub struct SettingsView {
    pub id: TournamentId,
    general: GeneralSettings,
    pairings: PairingsSettings,
    scoring: ScoringSettings,
    to_change: TournamentSettingsTreeBuilder,
}

impl Component for SettingsView {
    type Message = SettingsMessage;
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let emitter = ctx.link().callback(SettingsMessage::Setting);
        SettingsView {
            id: ctx.props().id,
            general: GeneralSettings::new(emitter),
            pairings: Default::default(),
            scoring: Default::default(),
            to_change: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SettingsMessage::Setting(setting) => self.to_change.add_setting(setting),
        };
        false
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
