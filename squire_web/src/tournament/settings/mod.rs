use std::collections::HashSet;

use yew::prelude::*;

use squire_sdk::{
    accounts::TournamentSettingsTree, client::state::ClientState,
    model::settings::TournamentSetting, tournaments::{TournamentId, TournamentPreset},
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
    Submitted,
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
    current: TournamentSettingsTree,
    to_change: TournamentSettingsTree,
}

impl Component for SettingsView {
    type Message = SettingsMessage;
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let emitter = ctx.link().callback(SettingsMessage::Setting);
        let id = ctx.props().id;
        let tree = CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&id, |tourn| tourn.settings())
            .unwrap_or_else(|| TournamentSettingsTree::new(TournamentPreset::Swiss));
        SettingsView {
            id,
            general: GeneralSettings::new(emitter),
            pairings: Default::default(),
            scoring: Default::default(),
            current: tree.clone(),
            to_change: tree,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SettingsMessage::Setting(setting) => {
                self.to_change.update(setting).is_ok();
                let count = self.to_change.diff(&self.current).count();
                web_sys::console::log_1(&format!("Different settings: {count}").into());
                false
            }
            SettingsMessage::Submitted => {
                /* Diff the current and to_change lists of settings
                 * Create a Vec of the changed settings
                 *              ^^^^^^^^^^^^^^^^^^ Look different for you
                 *
                 * Submit bulk update to CLIENT
                 * Create listener to follow the UpdateTracker
                 * ^^^^^^^^^^^^^^^^^^ Look same between Rounds and Settings pages
                 */
                todo!()
            }
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
                        { self.general.view(&self.current.general) }
                        { self.pairings.view(tourn) }
                        { self.scoring.view(tourn) }
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
