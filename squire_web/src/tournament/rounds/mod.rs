use chrono::{DateTime, Utc};
use squire_sdk::{
    model::{
        identifiers::{AdminId, RoundIdentifier},
        rounds::{RoundId, RoundStatus},
    },
<<<<<<< Updated upstream
    tournaments::TournamentId,
=======
    tournaments::{TournamentId, OpResult}, client::update::UpdateTracker,
>>>>>>> Stashed changes
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

pub mod creator;
pub mod input;
pub mod roundchangesbuffer;
pub mod roundresultticker;
pub mod roundconfirmationticker;
pub mod scroll;
pub mod selected;
pub use creator::*;
pub use input::*;
pub use roundresultticker::*;
pub use roundconfirmationticker::*;
pub use scroll::*;
pub use selected::*;

use self::_RoundsFilterProps::admin_id;

use super::spawn_update_listener;

#[derive(Debug, PartialEq, Properties)]
pub struct RoundsFilterProps {
    pub id: TournamentId,
    pub admin_id: AdminId,
    pub send_op_result: Callback<OpResult>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RoundsViewMessage {
    FilterInput(RoundFilterInputMessage),
    RoundScroll(RoundScrollMessage),
    SelectedRound(SelectedRoundMessage),
    ReQuery,
}

pub struct RoundsView {
    pub id: TournamentId,
    pub admin_id: AdminId,
    input: RoundFilterInput,
    scroll: RoundScroll,
    selected: SelectedRound,
    send_op_result: Callback<OpResult>,
}

impl Component for RoundsView {
    type Message = RoundsViewMessage;
    type Properties = RoundsFilterProps;

    fn create(ctx: &Context<Self>) -> Self {
<<<<<<< Updated upstream
        let id = ctx.props().id;
        let aid = ctx.props().admin_id;
        spawn_update_listener(ctx, RoundsViewMessage::ReQuery);
=======
        let RoundsFilterProps { id, admin_id, send_op_result } = ctx.props();
>>>>>>> Stashed changes
        Self {
            id,
            send_op_result,
            input: RoundFilterInput::new(ctx.link().callback(RoundsViewMessage::FilterInput)),
            scroll: RoundScroll::new(ctx, id),
            admin_id: aid,
            selected: SelectedRound::new(ctx, id, aid),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RoundsViewMessage::FilterInput(msg) => self.input.update(msg),
            RoundsViewMessage::RoundScroll(msg) => self.scroll.update(msg),
            RoundsViewMessage::SelectedRound(msg) => self.selected.update(ctx, msg),
            RoundsViewMessage::ReQuery => {
                spawn_update_listener(ctx, RoundsViewMessage::ReQuery);
                self.scroll.requery(ctx);
                self.selected.try_requery_existing(ctx);
                false
            }
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
