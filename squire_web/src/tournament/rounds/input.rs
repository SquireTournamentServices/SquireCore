use squire_sdk::model::{
    identifiers::RoundIdentifier,
    rounds::{Round, RoundStatus},
};
use web_sys::HtmlInputElement;

use yew::prelude::*;

use crate::utils::TextInput;

#[derive(PartialEq, Properties)]
pub struct RoundFilterInputProps {
    pub process: Callback<RoundFilterReport>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RoundFilterInputMessage {
    RoundNumber(String),
    RoundStatus(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RoundFilterReport {
    ident: Option<RoundIdentifier>,
    status: Option<RoundStatus>,
}

pub struct RoundFilterInput {
    ident: Option<RoundIdentifier>,
    status: Option<RoundStatus>,
    process: Callback<RoundFilterInputMessage>,
}

impl RoundFilterInput {
    pub fn get_report(&self) -> RoundFilterReport {
        RoundFilterReport {
            ident: self.ident,
            status: self.status,
        }
    }
}

impl RoundFilterInput {
    pub fn new(process: Callback<RoundFilterInputMessage>) -> Self {
        Self {
            ident: None,
            status: None,
            process,
        }
    }

    pub fn update(&mut self, msg: RoundFilterInputMessage) -> bool {
        match msg {
            RoundFilterInputMessage::RoundNumber(s) => {
                let ident = s.parse().ok().map(RoundIdentifier::Number);
                let digest = self.ident != ident;
                self.ident = ident;
                digest
            }
            RoundFilterInputMessage::RoundStatus(s) => {
                let status = s.parse().ok();
                let digest = self.status != status;
                self.status = status;
                digest
            }
        }
    }

    pub fn view(&self) -> Html {
        let number = self.process.clone();
        let number = Callback::from(move |s| number.emit(RoundFilterInputMessage::RoundNumber(s)));
        let status = self.process.clone();
        let status = Callback::from(move |s| status.emit(RoundFilterInputMessage::RoundStatus(s)));
        html! {
            <div>
                <div class="m-1">
                    <TextInput label = { "Round Number:" } process = { number }/>
                </div>
                <div class="m-1">
                    <TextInput label = { "Round Status:" } process = { status }/>
                </div>
            </div>
        }
    }
}

impl RoundFilterReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn matches(&self, rnd: &Round) -> bool {
        web_sys::console::log_1(&format!("Checking round #{:?}", rnd.match_number).into());
        self.status
            .as_ref()
            .map(|status| rnd.status == *status)
            .unwrap_or(true)
            && self
                .ident
                .as_ref()
                .map(|ident| round_ident_soft_matches(ident, rnd))
                .unwrap_or(true)
    }
}

fn round_ident_soft_matches(ident: &RoundIdentifier, rnd: &Round) -> bool {
    match ident {
        RoundIdentifier::Number(num) => {
            let temp_num = num.to_string();
            let temp_rnd = rnd.match_number.to_string();
            temp_rnd.contains(&temp_num)
        }
        RoundIdentifier::Table(num) => {
            let temp_num = num.to_string();
            let temp_rnd = rnd.table_number.to_string();
            temp_rnd.contains(&temp_num)
        }
        RoundIdentifier::Id(_) => true,
    }
}
