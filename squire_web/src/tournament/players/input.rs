use std::borrow::Cow;

use squire_sdk::model::{
    identifiers::{AdminId, TournamentId},
    operations::{JudgeOp, TournOp},
    players::PlayerStatus,
};
use yew::prelude::*;

use super::{PlayerSummary};
use crate::{utils::TextInput, CLIENT, tournament::viewer_component::{WrapperState}};

#[derive(PartialEq, Properties)]
pub struct PlayerFilterInputProps {}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerFilterInputMessage {
    PlayerName(String),
    PlayerStatus(String),
    GuestName(String),
    SubmitGuest,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct PlayerFilterReport {
    name: Option<String>,
    status: Option<PlayerStatus>,
}

pub struct PlayerFilterInput {
    name: Option<String>,
    status: Option<PlayerStatus>,
    guest_name: Option<String>,
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
    pub fn new(
        process: Callback<PlayerFilterInputMessage>,
        _id: TournamentId,
        _admin_id: AdminId,
    ) -> Self {
        Self {
            name: None,
            status: None,
            guest_name: None,
            process,
        }
    }

    pub fn update(&mut self, msg: PlayerFilterInputMessage, state: &WrapperState) -> bool {
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
            PlayerFilterInputMessage::GuestName(name) => {
                self.guest_name = Some(name);
                true
            }
            PlayerFilterInputMessage::SubmitGuest => {
                if self.guest_name.is_none() {
                    return false;
                };
                let _tracker = CLIENT.get().unwrap().update_tourn(
                    state.t_id,
                    TournOp::JudgeOp(
                        state.a_id.into(),
                        JudgeOp::RegisterGuest(self.guest_name.as_ref().unwrap().clone()),
                    ),
                );
                false
            }
        }
    }

    pub fn view(&self) -> Html {
        let number = self.process.clone();
        let number = Callback::from(move |s| number.emit(
            PlayerFilterInputMessage::PlayerName(s)
        ));
        let status = self.process.clone();
        let status = Callback::from(move |s| status.emit(
            PlayerFilterInputMessage::PlayerStatus(s)
        ));
        let guest_name = self.process.clone();
        let guest_name =
            Callback::from(move |s| guest_name.emit(
            PlayerFilterInputMessage::GuestName(s)
        ));
        let cb = self.process.clone();
        let submit_guest = move |_| { cb.emit(
            PlayerFilterInputMessage::SubmitGuest
        )};
        html! {
            <div class="row">
                <div class="col">
                    <h3>{"Search"}</h3>
                    <div class="my-1">
                        <TextInput label = {Cow::from("Player Name:")} process = { number } />
                    </div>
                    <div class="my-1">
                        <TextInput label = {Cow::from("Player Status:")} process = { status } />
                    </div>
                </div>
                <div class="col">
                    <h3>{"Add Guest Player"}</h3>
                    <div class="my-1">
                        <TextInput label = {Cow::from("Guest Name:")} process = { guest_name } />
                        <button onclick={submit_guest} >{"Submit"}</button>
                    </div>
                </div>
            </div>
        }
    }
}

impl PlayerFilterReport {
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
