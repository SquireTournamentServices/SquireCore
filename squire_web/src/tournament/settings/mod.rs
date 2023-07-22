use squire_sdk::model::{
    identifiers::{AdminId, TournamentId},
    operations::{OpResult, TournOp},
    settings::{
        GeneralSettingsTree, PairingSettingsTree, ScoringSettingsTree, SettingsTree,
        TournamentSetting, TournamentSettingsTree,
    },
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod general;
mod pairings;
mod panel;
mod scoring;

use general::*;
use pairings::*;
use scoring::*;

use super::spawn_update_listener;
use crate::CLIENT;

#[derive(Debug, PartialEq, Eq)]
pub enum SettingsMessage {
    Setting(TournamentSetting),
    QueryReady(Box<TournamentSettingsTree>),
    ReQuery,
    Submitted,
}

#[derive(Debug, Properties, PartialEq)]
pub struct SettingsProps {
    pub id: TournamentId,
    pub admin_id: AdminId,
    pub send_op_result: Callback<OpResult>,
}

pub struct SettingsView {
    pub id: TournamentId,
    admin_id: AdminId,
    send_op_result: Callback<OpResult>,
    general: GeneralSettings,
    pairings: PairingsSettings,
    scoring: ScoringSettings,
}

impl SettingsView {
    fn send_query(&self, ctx: &Context<Self>) {
        let id = self.id;
        ctx.link().send_future(async move {
            let settings = CLIENT
                .get()
                .unwrap()
                .query_tourn(id, |tourn| tourn.settings())
                .process()
                .await
                .unwrap_or_else(TournamentSettingsTree::new);
            SettingsMessage::QueryReady(Box::new(settings))
        })
    }
}

impl Component for SettingsView {
    type Message = SettingsMessage;
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        spawn_update_listener(ctx, SettingsMessage::ReQuery);
        let emitter = ctx.link().callback(SettingsMessage::Setting);
        let SettingsProps {
            id,
            send_op_result,
            admin_id,
        } = ctx.props();
        let digest = SettingsView {
            id: *id,
            admin_id: *admin_id,
            send_op_result: send_op_result.clone(),
            general: GeneralSettings::new(emitter.clone(), GeneralSettingsTree::new()),
            pairings: PairingsSettings::new(emitter.clone(), PairingSettingsTree::new()),
            scoring: ScoringSettings::new(emitter, ScoringSettingsTree::new()),
        };
        digest.send_query(ctx);
        digest
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SettingsMessage::Setting(setting) => match setting {
                TournamentSetting::GeneralSetting(setting) => self.general.update(setting),
                TournamentSetting::PairingSetting(setting) => self.pairings.update(setting),
                TournamentSetting::ScoringSetting(_) => false,
            },
            SettingsMessage::Submitted => {
                let client = CLIENT.get().unwrap();
                let iter = self
                    .general
                    .get_changes()
                    .chain(self.scoring.get_changes())
                    .chain(self.pairings.get_changes())
                    .map(|s| TournOp::AdminOp(self.admin_id, s.into()))
                    .collect::<Vec<_>>();
                let tracker = client.bulk_update(self.id, iter);
                let send_op_result = self.send_op_result.clone();
                spawn_local(async move { send_op_result.emit(tracker.process().await.unwrap()) });
                false
            }
            SettingsMessage::ReQuery => {
                spawn_update_listener(ctx, SettingsMessage::ReQuery);
                self.send_query(ctx);
                false
            }
            SettingsMessage::QueryReady(settings) => {
                let TournamentSettingsTree {
                    general,
                    pairing,
                    scoring,
                } = *settings;
                let emitter = ctx.link().callback(SettingsMessage::Setting);
                self.general = GeneralSettings::new(emitter.clone(), general);
                self.scoring = ScoringSettings::new(emitter.clone(), scoring);
                self.pairings = PairingsSettings::new(emitter, pairing);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| SettingsMessage::Submitted);
        html! {
            <div>
                <button onclick = { submit }> { "Update Settings"} </button>
                { self.general.view() }
                { self.pairings.view() }
                { self.scoring.view() }
            </div>
        }
    }
}
