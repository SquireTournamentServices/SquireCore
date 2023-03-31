use yew::prelude::*;

use squire_sdk::{model::settings::TournamentSetting, tournaments::Tournament};

use super::{panel::SettingPanel, SettingsMessage};

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
        let make_panel = |label, item| SettingPanel::new(label, item, emitter.clone());
        SettingPanel::new("Starting table #", TournamentSetting::StartingTableNumber, emitter.clone());
        Self {
            starting_table: make_panel("Starting table #", TournamentSetting::StartingTableNumber),
            use_table_num: make_panel("Use table #", TournamentSetting::StartingTableNumber),
            min_decks: make_panel("Min deck count", TournamentSetting::StartingTableNumber),
            max_decks: make_panel("Max deck count", TournamentSetting::StartingTableNumber),
            require_checkin: make_panel("Require check in", TournamentSetting::StartingTableNumber),
            require_decks: make_panel("Require deck reg.", TournamentSetting::StartingTableNumber),
            round_length: make_panel("Round length", TournamentSetting::StartingTableNumber),
        }
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        html! {
            <div>
                <h2>{ "General Settings:" }</h2>
                <p>{ format!("Status: {}", tourn.status) }</p>
                <p>{ format!("Registration open: {}", tourn.reg_open) }</p>
                { self.starting_table.view(tourn.round_reg.starting_table) }
                { self.use_table_num.view(tourn.use_table_number) }
                { self.min_decks.view(tourn.min_deck_count) }
                { self.max_decks.view(tourn.max_deck_count) }
                { self.require_checkin.view(tourn.require_check_in) }
                { self.require_decks.view(tourn.require_deck_reg) }
                { self.round_length.view(tourn.round_reg.length.as_secs()/60) }
            </div>
        }
    }
}
