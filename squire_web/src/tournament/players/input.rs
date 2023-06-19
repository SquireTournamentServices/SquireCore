use std::borrow::Cow;

use squire_sdk::model::{
    identifiers::RoundIdentifier,
    players::{Player, PlayerStatus},
    rounds::RoundStatus,
};
use web_sys::HtmlInputElement;

use yew::prelude::*;

use crate::utils::TextInput;

use super::PlayerSummary;

#[derive(PartialEq, Properties)]
pub struct PlayerFilterInputProps {
    pub process: Callback<PlayerFilterReport>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerFilterInputMessage {
    PlayerName(String),
    PlayerStatus(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct PlayerFilterReport {
    name: Option<String>,
    status: Option<PlayerStatus>,
}

pub struct PlayerFilterInput {
    name: Option<String>,
    status: Option<PlayerStatus>,
    process: Callback<PlayerFilterInputMessage>,
}

impl PlayerFilterInput {
    pub fn get_report(&self) -> PlayerFilterReport {
        PlayerFilterReport {
            name: self.name.clone(),
            status: self.status,
        }
    }
}

impl PlayerFilterInput {
    pub fn new(process: Callback<PlayerFilterInputMessage>) -> Self {
        Self {
            name: None,
            status: None,
            process,
        }
    }

    pub fn update(&mut self, msg: PlayerFilterInputMessage) -> bool {
        match msg {
            PlayerFilterInputMessage::PlayerName(name) => {
                self.name = Some(name);
                true
            }
            PlayerFilterInputMessage::PlayerStatus(s) => {
                let status = s.parse().ok();
                let digest = self.status != status;
                self.status = status;
                digest
            }
        }
    }

    pub fn view(&self) -> Html {
        let number = self.process.clone();
        let number = Callback::from(move |s| number.emit(PlayerFilterInputMessage::PlayerName(s)));
        let status = self.process.clone();
        let status =
            Callback::from(move |s| status.emit(PlayerFilterInputMessage::PlayerStatus(s)));
        html! {
            <div>
                <div class="my-1">
                    <TextInput label = {Cow::from("Player Name:")} process = { number }/>
                </div>
                <div class="my-1">
                    <TextInput label = {Cow::from("Player Status:")} process = { status }/>
                </div>
            </div>
        }
    }
}

impl PlayerFilterReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn matches(&self, plyr: &PlayerSummary) -> bool {
        self.status
            .as_ref()
            .map(|status| plyr.status == *status)
            .unwrap_or(true)
            && self
                .name
                .as_ref()
                .map(|name| plyr.name.contains(name))
                .unwrap_or(true)
    }
}
