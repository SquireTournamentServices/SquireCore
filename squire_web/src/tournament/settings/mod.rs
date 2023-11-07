use squire_sdk::{
    model::{
        identifiers::TournamentId,
        operations::AdminOp,
        settings::{
            GeneralSettingsTree, PairingSettingsTree, ScoringSettingsTree, SettingsTree,
            TournamentSetting, TournamentSettingsTree,
        },
    },
    sync::TournamentManager,
};
use yew::prelude::*;

mod general;
mod pairings;
mod panel;
mod scoring;

use general::*;
use pairings::*;
use scoring::*;

use crate::{
    tournament::{
        InteractionResponse, Op, TournQuery, TournViewerComponent, TournViewerComponentWrapper,
        WrapperMessage, WrapperState,
    },
    CLIENT,
};

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

impl SettingsView {}

impl TournViewerComponent for SettingsView {
    type Properties = SettingsProps;
    type InteractionMessage = SettingsMessage;
    type QueryMessage = SettingsQueryMessage;

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self {
        let emitter = ctx
            .link()
            .callback(|val| WrapperMessage::Interaction(SettingsMessage::Setting(val)));

        SettingsView {
            id: state.t_id,
            general: GeneralSettings::new(emitter.clone(), GeneralSettingsTree::new()),
            pairings: PairingsSettings::new(emitter.clone(), PairingSettingsTree::new()),
            scoring: ScoringSettings::new(emitter, ScoringSettingsTree::new()),
        }
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
                true
            }
        }
    }

    fn interaction(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _msg: Self::InteractionMessage,
        state: &WrapperState,
    ) -> InteractionResponse<Self> {
        match _msg {
            SettingsMessage::Setting(setting) => match setting {
                TournamentSetting::GeneralSetting(setting) => self.general.update(setting).into(),
                TournamentSetting::PairingSetting(setting) => self.pairings.update(setting).into(),
                TournamentSetting::ScoringSetting(setting) => self.scoring.update(setting).into(),
            },
            SettingsMessage::StartTourn => state.op_response(vec![Op::Admin(AdminOp::Start)]),
            SettingsMessage::Submitted => {
                let _client = CLIENT.get().unwrap();
                let iter = self
                    .general
                    .get_changes()
                    .chain(self.scoring.get_changes())
                    .chain(self.pairings.get_changes())
                    .map(|s| Op::Admin(s.into()))
                    .collect::<Vec<_>>();
                state.op_response(iter)
            }
        }
    }

    fn query(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> TournQuery<Self::QueryMessage> {
        let q_func = |tourn: &TournamentManager| {
            let tree = tourn.settings();
            SettingsQueryMessage::AllDataReady(Box::new(tree))
        };
        Box::new(q_func)
    }

    fn v_view(
        &self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> Html {
        let submit = ctx
            .link()
            .callback(|_| WrapperMessage::Interaction(SettingsMessage::Submitted));
        let start = ctx
            .link()
            .callback(|_| WrapperMessage::Interaction(SettingsMessage::StartTourn));
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
