use squire_sdk::{
    client::state::ClientState,
    model::{identifiers::RoundIdentifier, rounds::RoundStatus},
    players::PlayerId,
    tournaments::TournamentId,
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

#[derive(Debug, PartialEq, Properties)]
pub struct PlayerFilterProps {
    pub id: TournamentId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerFilterMessage {
    PlayerSelected(PlayerId),
    FilterInput(PlayerFilterInputMessage),
}

pub struct PlayerView {
    pub id: TournamentId,
    input: PlayerFilterInput,
    scroll: PlayerScroll,
    selected: SelectedPlayer,
}

impl Component for PlayerView {
    type Message = PlayerFilterMessage;
    type Properties = PlayerFilterProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            id: ctx.props().id,
            input: PlayerFilterInput::new(ctx.link().callback(PlayerFilterMessage::FilterInput)),
            scroll: PlayerScroll::new(ctx.link().callback(PlayerFilterMessage::PlayerSelected)),
            selected: SelectedPlayer::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        web_sys::console::log_1(&format!("New filter message: {msg:?}").into());
        match msg {
            PlayerFilterMessage::FilterInput(msg) => {
                if self.input.update(msg) {
                    self.scroll.update(self.input.get_report())
                } else {
                    false
                }
            }
            PlayerFilterMessage::PlayerSelected(p_id) => {
                self.selected.update(Some(p_id))
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |t| {
                let process = ctx.link().callback(PlayerFilterMessage::FilterInput);
                html! {
                    <div>
                        { self.input.view() }
                        <div class="row">
                            <div class="col">
                                <div class="overflow-auto player-scroll-box">
                                    { self.scroll.view(t) }
                                </div>
                            </div>
                            <div class="col">
                                { self.selected.view(t) }
                            </div>
                        </div>
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
