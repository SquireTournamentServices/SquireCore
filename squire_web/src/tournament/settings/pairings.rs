use squire_sdk::model::settings::{
    CommonPairingSetting, FluidPairingSetting, FluidPairingSettingsTree, PairingCommonSettingsTree,
    PairingSetting, PairingSettingsTree, PairingStyleSetting, PairingStyleSettingsTree,
    SettingsTree, SwissPairingSetting, SwissPairingSettingsTree, TournamentSetting,
};
use yew::prelude::*;

use super::panel::{make_panel, SettingPanel};

pub struct PairingsSettings {
    common: CommonPairingSection,
    style: PairingStyleSection,
}

impl PairingsSettings {
    pub(crate) fn get_changes(&self) -> impl Iterator<Item = TournamentSetting> {
        self.common
            .get_changes()
            .chain(self.style.get_changes())
            .map(Into::into)
    }
}

impl PairingsSettings {
    pub fn new(emitter: Callback<TournamentSetting>, settings: PairingSettingsTree) -> Self {
        let PairingSettingsTree { common, style } = settings;
        Self {
            common: CommonPairingSection::new(common, emitter.clone()),
            style: PairingStyleSection::new(emitter, style),
        }
    }

    pub fn update(&mut self, setting: PairingSetting) -> bool {
        match setting {
            PairingSetting::Common(setting) => {
                self.common.update(setting);
            }
            PairingSetting::Style(setting) => {
                self.style.update(setting);
            }
        };
        false
    }

    pub fn view(&self) -> Html {
        html! {
            <div>
                <h2>{ "Pairings Settings:" }</h2>
                { self.common.view() }
                { self.style.view() }
            </div>
        }
    }
}

struct CommonPairingSection {
    match_size: SettingPanel,
    repair_tolerance: SettingPanel,
    algorithm: SettingPanel,
    current: PairingCommonSettingsTree,
    to_change: PairingCommonSettingsTree,
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

impl CommonPairingSection {
    fn new(common: PairingCommonSettingsTree, emitter: Callback<TournamentSetting>) -> Self {
        Self {
            match_size: make_panel(&emitter, "Match size", CommonPairingSetting::MatchSize),
            repair_tolerance: make_panel(
                &emitter,
                "Repair Tolerance",
                CommonPairingSetting::RepairTolerance,
            ),
            algorithm: make_panel(
                &emitter,
                "Pairing Algorithm",
                CommonPairingSetting::Algorithm,
            ),
            current: common.clone(),
            to_change: common,
        }
    }

    fn get_changes(&self) -> impl Iterator<Item = PairingSetting> {
        self.to_change.diff(&self.current).map(Into::into)
    }

    fn update(&mut self, setting: CommonPairingSetting) {
        let _ = self.to_change.update(setting);
    }

    fn view(&self) -> Html {
        html! {
            <>
                <h3>{ "General Pairing Settings:" }</h3>
                <p>{ self.match_size.view(self.current.match_size) }</p>
                <p>{ self.repair_tolerance.view(self.current.repair_tolerance) }</p>
                <p>{ self.algorithm.view(self.current.algorithm) }</p>
            </>
        }
    }
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

    fn get_changes(&self) -> Box<dyn Iterator<Item = PairingSetting>> {
        match self {
            PairingStyleSection::Swiss(settings) => Box::new(settings.get_changes()),
            PairingStyleSection::Fluid(settings) => Box::new(settings.get_changes()),
        }
    }

    fn update(&mut self, setting: PairingStyleSetting) {
        match (self, setting) {
            (PairingStyleSection::Swiss(style), PairingStyleSetting::Swiss(setting)) => {
                style.update(setting)
            }
            (PairingStyleSection::Fluid(style), PairingStyleSetting::Fluid(setting)) => {
                style.update(setting)
            }
            _ => {}
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
            to_change: settings,
            do_checkins: make_panel(&emitter, "Do checkins?", SwissPairingSetting::DoCheckIns),
        }
    }

    fn get_changes(&self) -> impl Iterator<Item = PairingSetting> {
        self.to_change.diff(&self.current).map(Into::into)
    }

    fn update(&mut self, setting: SwissPairingSetting) {
        let _ = self.to_change.update(setting);
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h3>{ "Swiss Pairing Settings:" }</h3>
                <p>{ self.do_checkins.view(self.current.do_checkins) }</p>
            </div>
        }
    }
}

impl FluidPairingSection {
    fn new(_emitter: Callback<TournamentSetting>, settings: FluidPairingSettingsTree) -> Self {
        Self {
            current: settings.clone(),
            to_change: settings,
        }
    }

    fn get_changes(&self) -> impl Iterator<Item = PairingSetting> {
        self.to_change.diff(&self.current).map(Into::into)
    }

    fn update(&mut self, setting: FluidPairingSetting) {
        let _ = self.to_change.update(setting);
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h3>{ "Fluid Pairing Settings:" }</h3>
            </div>
        }
    }
}
