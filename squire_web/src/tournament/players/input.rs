use squire_sdk::model::{identifiers::RoundIdentifier, rounds::RoundStatus};
use web_sys::HtmlInputElement;

use yew::prelude::*;

use crate::utils::TextInput;

#[derive(PartialEq, Properties)]
pub struct PlayerFilterInputProps {
    pub process: Callback<PlayerFilterReport>,
}

pub enum PlayerFilterInputMessage {
    RoundNumber(String),
    RoundStatus(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct PlayerFilterReport {
    ident: Option<RoundIdentifier>,
    status: Option<RoundStatus>,
}

pub struct PlayerFilterInput {
    ident: Option<RoundIdentifier>,
    status: Option<RoundStatus>,
    process: Callback<PlayerFilterInputMessage>,
}

impl PlayerFilterInput {
    pub fn get_report(&self) -> PlayerFilterReport {
        PlayerFilterReport {
            ident: self.ident,
            status: self.status,
        }
    }
}

impl PlayerFilterInput {
    pub fn new(process: Callback<PlayerFilterInputMessage>) -> Self {
        Self {
            ident: None,
            status: None,
            process,
        }
    }

    pub fn update(&mut self, msg: PlayerFilterInputMessage) -> bool {
        match msg {
            PlayerFilterInputMessage::RoundNumber(s) if let Ok(num) = s.parse() => {
                self.ident = Some(RoundIdentifier::Number(num));
                true
            },
            PlayerFilterInputMessage::RoundStatus(s) if let Ok(status) = s.parse() => {
                self.status = Some(status);
                true
            },
            _ => {
                false
            }
        }
    }

    pub fn view(&self) -> Html {
        let number = self.process.clone();
        let number = Callback::from(move |s| number.emit(PlayerFilterInputMessage::RoundNumber(s)));
        let status = self.process.clone();
        let status = Callback::from(move |s| status.emit(PlayerFilterInputMessage::RoundStatus(s)));
        html! {
            <div>
                <TextInput label = { "Round Number:" } process = { number }/>
                <TextInput label = { "Round Status:" } process = { status }/>
            </div>
        }
    }
}
