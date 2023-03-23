use yew::prelude::*;

use squire_sdk::tournaments::Tournament;

#[derive(Debug, Default)]
pub struct GeneralSettings {}

impl GeneralSettings {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        html! {
            <div>
                <h2>{ "General Settings:" }</h2>
                <p>{ format!("Status: {}", tourn.status) }</p>
                <p>{ format!("Registration open: {}", tourn.reg_open) }</p>
                <p>{ format!("Startin table number: {}", tourn.round_reg.starting_table) }</p>
                <p>{ format!("Use table numbers: {}", tourn.use_table_number) }</p>
                <p>{ format!("Min. deck count: {}", tourn.min_deck_count) }</p>
                <p>{ format!("Max. deck count: {}", tourn.max_deck_count) }</p>
                <p>{ format!("Require check in: {}", tourn.require_check_in) }</p>
                <p>{ format!("Require deck reg.: {}", tourn.require_deck_reg) }</p>
                <p>{ format!("Round length: {} mins.", tourn.round_reg.length.as_secs()/60) }</p>
            </div>
        }
    }
}
