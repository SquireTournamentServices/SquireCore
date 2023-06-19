use std::{str::FromStr, time::Duration};

use squire_sdk::{
    model::settings::{GeneralSetting, GeneralSettingsTree, TournamentSetting},
    tournaments::Tournament,
};
use yew::prelude::*;

use super::{
    panel::{make_panel, SettingPanel},
    SettingsMessage,
};

pub struct GeneralSettings {
    starting_table: SettingPanel,
    use_table_num: SettingPanel,
    min_decks: SettingPanel,
    max_decks: SettingPanel,
    require_checkin: SettingPanel,
    require_decks: SettingPanel,
    round_length: SettingPanel,
    current: GeneralSettingsTree,
    to_change: GeneralSettingsTree,
}

impl GeneralSettings {
    pub fn new(emitter: Callback<TournamentSetting>, tree: GeneralSettingsTree) -> Self {
        use GeneralSetting::*;
        Self {
            starting_table: make_panel(&emitter, "Starting table #", StartingTableNumber),
            use_table_num: make_panel(&emitter, "Use table #", UseTableNumbers),
            min_decks: make_panel(&emitter, "Min deck count", MinDeckCount),
            max_decks: make_panel(&emitter, "Max deck count", MaxDeckCount),
            require_checkin: make_panel(&emitter, "Require check in", RequireCheckIn),
            require_decks: make_panel(&emitter, "Require deck reg.", RequireDeckReg),
            round_length: make_panel(&emitter, "Round length", |l: u64| {
                RoundLength(Duration::from_secs(l * 60))
            }),
            current: tree.clone(),
            to_change: tree,
        }
    }

    pub fn update(&mut self, setting: GeneralSetting) -> bool {
        self.to_change.update(setting);
        false
    }

    pub fn view(&self) -> Html {
        html! {
            <div>
                <h2>{ "General Settings:" }</h2>
                { self.starting_table.view(self.current.starting_table_number) }
                { self.use_table_num.view(self.current.use_table_number) }
                { self.min_decks.view(self.current.min_deck_count) }
                { self.max_decks.view(self.current.max_deck_count) }
                { self.require_checkin.view(self.current.require_check_in) }
                { self.require_decks.view(self.current.require_deck_reg) }
                { self.round_length.view(self.current.round_length.as_secs()/60) }
            </div>
        }
    }
}
