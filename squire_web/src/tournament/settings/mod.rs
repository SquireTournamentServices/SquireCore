use squire_sdk::model::{
    identifiers::{TournamentId},
    operations::{AdminOp},
    settings::{
        GeneralSettingsTree, PairingSettingsTree, ScoringSettingsTree, SettingsTree,
        TournamentSetting, TournamentSettingsTree,
    },
};
use squire_sdk::sync::TournamentManager;
use yew::prelude::*;

mod general;
mod pairings;
mod panel;
mod scoring;

use general::*;
use pairings::*;
use scoring::*;

use crate::CLIENT;
use crate::tournament::viewer_component::{InteractionResponse, Op, TournQuery, TournViewerComponent, TournViewerComponentWrapper, WrapperMessage, WrapperState};

#[derive(Debug, PartialEq, Eq)]
pub enum SettingsMessage {
    Setting(TournamentSetting),
    Submitted,
    StartTourn,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SettingsQueryMessage {
    AllDataReady(Box<TournamentSettingsTree>),
}

#[derive(Debug, Properties, PartialEq)]
pub struct SettingsProps {}

pub struct SettingsView {
    pub id: TournamentId,
    general: GeneralSettings,
    pairings: PairingsSettings,
    scoring: ScoringSettings,
}

impl SettingsView {
    /*
    fn send_query(&self, ctx: &Context<Self>) {
        let id = self.id;
        ctx.link().send_future(async move {
            let settings = CLIENT
                .get()
                .unwrap()
                .query_tourn(id, |tourn| tourn.settings())
                .await
                .unwrap_or_else(TournamentSettingsTree::new);
            SettingsMessage::QueryReady(Box::new(settings))
        })
    }
     */
}

impl TournViewerComponent for SettingsView {
    type Properties = SettingsProps;
    type InteractionMessage = SettingsMessage;
    type QueryMessage = SettingsQueryMessage;

    /*
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
            SettingsMessage::StartTourn => {
                let op = TournOp::AdminOp(self.admin_id, AdminOp::Start);
                let tracker = CLIENT.get().unwrap().update_tourn(self.id, op);
                let send_op_result = self.send_op_result.clone();
                spawn_local(async move { send_op_result.emit(tracker.await.unwrap()) });
                false
            }
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
                spawn_local(async move { send_op_result.emit(tracker.await.unwrap()) });
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
                }: TournamentSettingsTree = *settings;
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
        let start = ctx.link().callback(|_| SettingsMessage::StartTourn);
        html! {
            <div>
                <button onclick = { submit }> { "Update Settings"} </button>
                <button onclick = { start }> { "Start Tourn"} </button>
                { self.general.view() }
                { self.pairings.view() }
                { self.scoring.view() }
            </div>
        }
    }
     */

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self {
        let emitter = ctx.link().callback( |val| WrapperMessage::Interaction(SettingsMessage::Setting(val)));
        let digest = SettingsView {
            id: state.t_id,
            general: GeneralSettings::new(emitter.clone(), GeneralSettingsTree::new()),
            pairings: PairingsSettings::new(emitter.clone(), PairingSettingsTree::new()),
            scoring: ScoringSettings::new(emitter, ScoringSettingsTree::new()),
        };
        digest
    }

    fn load_queried_data(&mut self, _msg: Self::QueryMessage, _state: &WrapperState) -> bool {
        match _msg {
            SettingsQueryMessage::AllDataReady(_data) => {
                let TournamentSettingsTree {
                    general,
                    pairing,
                    scoring,
                }: TournamentSettingsTree = *_data;
                self.general.load_settings(general);
                self.scoring.load_settings(scoring);
                self.pairings.load_settings(pairing);
                true.into()
            }
        }
    }

    fn interaction(&mut self, _ctx: &Context<TournViewerComponentWrapper<Self>>, _msg: Self::InteractionMessage, state: &WrapperState) -> InteractionResponse<Self> {
        match _msg {
            SettingsMessage::Setting(setting) => match setting {
                TournamentSetting::GeneralSetting(setting) => self.general.update(setting).into(),
                TournamentSetting::PairingSetting(setting) => self.pairings.update(setting).into(),
                TournamentSetting::ScoringSetting(setting) => self.scoring.update(setting).into()
            }
            SettingsMessage::StartTourn => {
                state.op_response(vec![Op::Admin(AdminOp::Start)])
            }
            SettingsMessage::Submitted => {
                let _client = CLIENT.get().unwrap();
                let iter = self
                    .general
                    .get_changes()
                    .chain(self.scoring.get_changes())
                    .chain(self.pairings.get_changes())
                    .map(|s| Op::Admin(s.into()) )
                    .collect::<Vec<_>>();
                state.op_response(iter)
            }
        }
    }

    fn query(&mut self, _ctx: &Context<TournViewerComponentWrapper<Self>>, _state: &WrapperState) -> TournQuery<Self::QueryMessage> {
        let q_func = |tourn: &TournamentManager| {
            let tree = tourn.settings();
            SettingsQueryMessage::AllDataReady(Box::new(tree))
        };
        Box::new(q_func)
    }

    fn v_view(&self, ctx: &Context<TournViewerComponentWrapper<Self>>, _state: &WrapperState) -> Html {
        let submit = ctx.link().callback(|_| WrapperMessage::Interaction(SettingsMessage::Submitted));
        let start = ctx.link().callback(|_| WrapperMessage::Interaction(SettingsMessage::StartTourn));
        html! {
            <div>
                <button onclick = { submit }> { "Update Settings"} </button>
                <button onclick = { start }> { "Start Tourn"} </button>
                { self.general.view() }
                { self.pairings.view() }
                { self.scoring.view() }
            </div>
        }
    }
}
