use std::collections::HashSet;

use yew::prelude::*;

use squire_sdk::{
    accounts::TournamentSettingsTree,
    model::settings::{
        GeneralSettingsTree, PairingSettingsTree, PairingStyleSettingsTree, ScoringSettingsTree,
        TournamentSetting,
    },
    tournaments::{TournamentId, TournamentPreset},
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
}

impl Component for SettingsView {
    type Message = SettingsMessage;
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let emitter = ctx.link().callback(SettingsMessage::Setting);
        let id = ctx.props().id;
        SettingsView {
            id,
            general: GeneralSettings::new(emitter.clone(), GeneralSettingsTree::new()),
            pairings: PairingsSettings::new(
                emitter,
                PairingSettingsTree::new(TournamentPreset::Swiss),
            ),
            scoring: ScoringSettings::new(ScoringSettingsTree::new(TournamentPreset::Swiss)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SettingsMessage::Setting(setting) => match setting {
                TournamentSetting::GeneralSetting(setting) => self.general.update(setting),
                TournamentSetting::PairingSetting(setting) => self.pairings.update(setting),
                TournamentSetting::ScoringSetting(_) => false,
            },
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
        html! {
            <div>
                { self.general.view() }
                { self.pairings.view() }
                { self.scoring.view() }
            </div>
        }
    }
}
