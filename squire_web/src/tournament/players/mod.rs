use squire_sdk::{
    model::{
        identifiers::PlayerIdentifier, identifiers::RoundIdentifier, players::PlayerId,
        rounds::RoundId, rounds::RoundStatus,
    },
    tournaments::{Tournament, TournamentId, TournamentManager},
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

pub mod creator;
pub mod input;
pub mod scroll;
pub mod selected;
pub use creator::*;
pub use input::*;
pub use scroll::*;
pub use selected::*;

use super::rounds::SelectedRound;

#[derive(Debug, PartialEq, Properties)]
pub struct PlayerViewProps {
    pub id: TournamentId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerViewMessage {
    FilterInput(PlayerFilterInputMessage),
    PlayerScroll(PlayerScrollMessage),
    SelectedPlayer(SelectedPlayerMessage),
}

pub struct PlayerView {
    pub id: TournamentId,
    input: PlayerFilterInput,
    scroll: PlayerScroll,
    selected: SelectedPlayer,
}

impl Component for PlayerView {
    type Message = PlayerViewMessage;
    type Properties = PlayerViewProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().id;
        Self {
            id,
            input: PlayerFilterInput::new(ctx.link().callback(PlayerViewMessage::FilterInput)),
            scroll: PlayerScroll::new(ctx, id),
            selected: SelectedPlayer::new(
                ctx.link().callback(PlayerViewMessage::SelectedPlayer),
                id,
            ),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlayerViewMessage::FilterInput(msg) => self.input.update(msg),
            PlayerViewMessage::SelectedPlayer(msg) => self.selected.update(ctx, msg),
            PlayerViewMessage::PlayerScroll(msg) => self.scroll.update(msg),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let report = self.input.get_report();
        let selected_pid = self.selected.id;
        html! {
            <div>
                { self.input.view() }
                <div class="d-flex flex-row my-4">
                    <div>
                        <div class="overflow-auto player-scroll-box">
                            { self.scroll.view(report) }
                        </div>
                    </div>
                    <div>
                        { self.selected.view() }
                    </div>
                </div>
            </div>
        }
    }
}

impl From<PlayerFilterInputMessage> for PlayerViewMessage {
    fn from(msg: PlayerFilterInputMessage) -> Self {
        Self::FilterInput(msg)
    }
}

impl From<SelectedPlayerMessage> for PlayerViewMessage {
    fn from(msg: SelectedPlayerMessage) -> Self {
        Self::SelectedPlayer(msg)
    }
}
