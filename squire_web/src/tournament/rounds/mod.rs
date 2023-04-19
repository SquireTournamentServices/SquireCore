use chrono::{DateTime, Utc};
use squire_sdk::{
    client::state::ClientState,
    model::{
        identifiers::RoundIdentifier,
        rounds::{RoundId, RoundStatus},
    },
    tournaments::TournamentId,
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

pub mod creator;
pub mod input;
pub mod roundresultticker;
pub mod scroll;
pub mod selected;
pub use creator::*;
pub use input::*;
pub use roundresultticker::*;
pub use scroll::*;
pub use selected::*;

#[derive(Debug, PartialEq, Properties)]
pub struct RoundsFilterProps {
    pub id: TournamentId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RoundsFilterMessage {
    RoundSelected(RoundId),
    FilterInput(RoundFilterInputMessage),
    TimerTick,
}

pub struct RoundsView {
    pub id: TournamentId,
    input: RoundFilterInput,
    scroll: RoundScroll,
    selected: Option<SelectedRound>,
}

impl Component for RoundsView {
    type Message = RoundsFilterMessage;
    type Properties = RoundsFilterProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            RoundsFilterMessage::TimerTick
        });
        Self {
            id: ctx.props().id,
            input: RoundFilterInput::new(ctx.link().callback(RoundsFilterMessage::FilterInput)),
            scroll: RoundScroll::new(ctx.link().callback(RoundsFilterMessage::RoundSelected)),
            selected: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RoundsFilterMessage::FilterInput(msg) => {
                if self.input.update(msg) {
                    self.scroll.update(self.input.get_report())
                } else {
                    false
                }
            }
            RoundsFilterMessage::RoundSelected(r_id) => {
                if self.selected.is_none() {
                    self.selected.insert(SelectedRound::new(r_id, self.id));
                }
                self.selected
                    .as_mut()
                    .map(|sr| sr.update(r_id))
                    .unwrap_or_default()
            }
            RoundsFilterMessage::TimerTick => {
                ctx.link().send_future(async move {
                    async_std::task::sleep(std::time::Duration::from_secs(1)).await;
                    RoundsFilterMessage::TimerTick
                });
                self.selected.is_some()
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |t| {
                let process = ctx.link().callback(RoundsFilterMessage::FilterInput);
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
                                { self.selected.as_ref().map(|sr| sr.view(t)).unwrap_or_default() }
                            </div>
                        </div>
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
