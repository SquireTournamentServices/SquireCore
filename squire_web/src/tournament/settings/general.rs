use std::{str::FromStr, time::Duration};

use yew::prelude::*;

use squire_sdk::{
    model::settings::{GeneralSetting, GeneralSettingsTree, TournamentSetting},
    tournaments::Tournament,
};

use super::{panel::{make_panel, SettingPanel}, SettingsMessage};

pub struct GeneralSettings {
    starting_table: SettingPanel,
    use_table_num: SettingPanel,
    min_decks: SettingPanel,
    max_decks: SettingPanel,
    require_checkin: SettingPanel,
    require_decks: SettingPanel,
    round_length: SettingPanel,
}

impl GeneralSettings {
    pub fn new(emitter: Callback<TournamentSetting>) -> Self {
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
        }
    }

    pub fn view(&self, settings: &GeneralSettingsTree) -> Html {
        html! {
            <div>
                <h2>{ "General Settings:" }</h2>
                { self.starting_table.view(settings.starting_table_number) }
                { self.use_table_num.view(settings.use_table_number) }
                { self.min_decks.view(settings.min_deck_count) }
                { self.max_decks.view(settings.max_deck_count) }
                { self.require_checkin.view(settings.require_check_in) }
                { self.require_decks.view(settings.require_deck_reg) }
                { self.round_length.view(settings.round_length.as_secs()/60) }
            </div>
        }
    }
}
