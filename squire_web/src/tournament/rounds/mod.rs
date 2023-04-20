use chrono::{DateTime, Utc};
use squire_sdk::{
    model::{
        identifiers::RoundIdentifier,
        rounds::{RoundId, RoundStatus},
    },
    tournaments::TournamentId,
};

use yew::prelude::*;
use roundchangesbuffer::*;

use crate::{utils::TextInput, CLIENT};

pub mod creator;
pub mod input;
pub mod roundresultticker;
pub mod scroll;
pub mod selected;
pub mod roundresultticker;
mod roundchangesbuffer;
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
pub enum RoundsViewMessage {
    FilterInput(RoundFilterInputMessage),
    RoundScroll(RoundScrollMessage),
    SelectedRound(SelectedRoundMessage),
}

pub struct RoundsView {
    pub id: TournamentId,
    input: RoundFilterInput,
    scroll: RoundScroll,
    selected: SelectedRound,
}

impl Component for RoundsView {
    type Message = RoundsViewMessage;
    type Properties = RoundsFilterProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().id;
        Self {
            id,
            input: RoundFilterInput::new(ctx.link().callback(RoundsViewMessage::FilterInput)),
            scroll: RoundScroll::new(ctx, id),
            selected: SelectedRound::new(ctx, id),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RoundsViewMessage::FilterInput(msg) => self.input.update(msg),
            RoundsViewMessage::RoundScroll(msg) => self.scroll.update(msg),
            RoundsViewMessage::SelectedRound(msg) => self.selected.update(ctx, msg),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { self.input.view() }
                <div class="d-flex flex-row my-4">
                    <div>
                        <div class="overflow-auto player-scroll-box">
                            { self.scroll.view(self.input.get_report()) }
                        </div>
                    </div>
                    <div> { self.selected.view() } </div>
                </div>
            </div>
        }
    }
}


impl From<RoundFilterInputMessage> for RoundsViewMessage {
    fn from(msg: RoundFilterInputMessage) -> Self {
        Self::FilterInput(msg)
    }
}

impl From<RoundScrollMessage> for RoundsViewMessage {
    fn from(msg: RoundScrollMessage) -> Self {
        Self::RoundScroll(msg)
    }
}

impl From<SelectedRoundMessage> for RoundsViewMessage {
    fn from(msg: SelectedRoundMessage) -> Self {
        Self::SelectedRound(msg)
    }
}
