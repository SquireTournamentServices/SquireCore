use yew::prelude::*;

use squire_sdk::{
    model::{
        pairings::{FluidPairings, PairingStyle, SwissPairings},
        settings::{
            CommonPairingSetting, FluidPairingSettingsTree, PairingCommonSettingsTree,
            PairingSettingsTree, PairingStyleSettingsTree, SwissPairingSetting,
            SwissPairingSettingsTree, TournamentSetting,
        },
    },
    tournaments::Tournament,
};

use super::panel::{make_panel, SettingPanel};

pub struct PairingsSettings {
    match_size: SettingPanel,
    repair_tolerance: SettingPanel,
    algorithm: SettingPanel,
    current: PairingCommonSettingsTree,
    to_change: PairingCommonSettingsTree,
    style: PairingStyleSection,
}

impl PairingsSettings {
    pub fn new(emitter: Callback<TournamentSetting>, settings: PairingSettingsTree) -> Self {
        let PairingSettingsTree { common, style } = settings;
        Self {
            match_size: make_panel(&emitter, "Match size: ", CommonPairingSetting::MatchSize),
            repair_tolerance: make_panel(
                &emitter,
                "Repair Tolerance: ",
                CommonPairingSetting::RepairTolerance,
            ),
            algorithm: make_panel(
                &emitter,
                "Pairing Algorithm: ",
                CommonPairingSetting::Algorithm,
            ),
            style: PairingStyleSection::new(emitter, style),
            current: common.clone(),
            to_change: common,
        }
    }

    pub fn view(&self) -> Html {
        html! {
            <div>
                <h2>{ "Pairings Settings:" }</h2>
                <p>{ format!("Match size: {}", self.current.match_size) }</p>
                <p>{ format!("Repair tolerance: {}", self.current.repair_tolerance) }</p>
                <p>{ format!("Algorithm: {}", self.current.algorithm) }</p>
                { self.style.view() }
            </div>
        }
    }
}

enum PairingStyleSection {
    Swiss(SwissPairingSection),
    Fluid(FluidPairingSection),
}

struct SwissPairingSection {
    do_checkins: SettingPanel,
    current: SwissPairingSettingsTree,
    to_change: SwissPairingSettingsTree,
}

struct FluidPairingSection {
    current: FluidPairingSettingsTree,
    to_change: FluidPairingSettingsTree,
}

impl PairingStyleSection {
    fn new(emitter: Callback<TournamentSetting>, style: PairingStyleSettingsTree) -> Self {
        match style {
            PairingStyleSettingsTree::Swiss(settings) => {
                Self::Swiss(SwissPairingSection::new(emitter, settings))
            }
            PairingStyleSettingsTree::Fluid(settings) => {
                Self::Fluid(FluidPairingSection::new(emitter, settings))
            }
        }
    }

    fn view(&self) -> Html {
        match self {
            PairingStyleSection::Swiss(style) => style.view(),
            PairingStyleSection::Fluid(style) => style.view(),
        }
    }
}

impl SwissPairingSection {
    fn new(emitter: Callback<TournamentSetting>, settings: SwissPairingSettingsTree) -> Self {
        Self {
            current: settings.clone(),
            to_change: settings.clone(),
            do_checkins: make_panel(&emitter, "Do checkins?", SwissPairingSetting::DoCheckIns),
        }
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h3>{ "Swiss Pairing Settings:" }</h3>
                <p>{ format!("Match size: {}", self.current.do_checkins) }</p>
            </div>
        }
    }
}

impl FluidPairingSection {
    fn new(emitter: Callback<TournamentSetting>, settings: FluidPairingSettingsTree) -> Self {
        Self {
            current: settings.clone(),
            to_change: settings,
        }
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h3>{ "Fluid Pairing Settings:" }</h3>
            </div>
        }
    }
}
