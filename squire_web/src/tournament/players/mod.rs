use squire_sdk::{
    model::{identifiers::PlayerIdentifier, rounds::RoundId},
    model::{identifiers::RoundIdentifier, rounds::RoundStatus},
    players::PlayerId,
    tournaments::{Tournament, TournamentId},
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
    PlayerSelected(PlayerId),
    FilterInput(PlayerFilterInputMessage),
    PlayerInfoSelected(SelectedPlayerInfo),
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
        Self {
            id: ctx.props().id,
            input: PlayerFilterInput::new(ctx.link().callback(PlayerViewMessage::FilterInput)),
            scroll: PlayerScroll::new(ctx.link().callback(PlayerViewMessage::PlayerSelected)),
            selected: SelectedPlayer::new(
                ctx.link().callback(PlayerViewMessage::PlayerInfoSelected),
            ),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlayerViewMessage::FilterInput(msg) => {
                if self.input.update(msg) {
                    self.scroll.update(self.input.get_report())
                } else {
                    false
                }
            }
            PlayerViewMessage::PlayerSelected(p_id) => self
                .selected
                .update(SelectedPlayerMessage::PlayerSelected(Some(p_id))),
            PlayerViewMessage::PlayerInfoSelected(spi) => self
                .selected
                .update(SelectedPlayerMessage::InfoSelected(Some(spi))),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |t| {
                html! {
                    <div>
                        { self.input.view() }
                        <div class="d-flex flex-row my-4">
                            <div>
                                <div class="overflow-auto player-scroll-box">
                                    { self.scroll.view(t) }
                                </div>
                            </div>
                            <div>
                                { self.selected.view(t) }
                            </div>
                        </div>
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
