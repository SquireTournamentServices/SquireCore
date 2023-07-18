use squire_sdk::{
    model::{
        identifiers::{AdminId},
        operations::AdminOp,
    },
    tournaments::{OpResult, TournOp, TournamentId},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    utils::{console_log},
    CLIENT,
};

pub mod creator;
pub mod input;
pub mod roundchangesbuffer;
pub mod roundconfirmationticker;
pub mod roundresultticker;
pub mod scroll;
pub mod selected;
pub use creator::*;
pub use input::*;
pub use roundconfirmationticker::*;
pub use roundresultticker::*;
pub use scroll::*;
pub use selected::*;

use super::spawn_update_listener;

#[derive(Debug, PartialEq, Properties, Clone)]
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
    BulkConfirmation,
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
        spawn_update_listener(ctx, RoundsViewMessage::ReQuery);
        let RoundsFilterProps {
            id,
            admin_id,
            send_op_result,
        } = ctx.props().clone();
        Self {
            id,
            send_op_result,
            input: RoundFilterInput::new(ctx.link().callback(RoundsViewMessage::FilterInput)),
            scroll: RoundScroll::new(ctx, id),
            admin_id,
            selected: SelectedRound::new(ctx, id, admin_id),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RoundsViewMessage::FilterInput(msg) => self.input.update(msg),
            RoundsViewMessage::RoundScroll(msg) => self.scroll.update(msg),
            RoundsViewMessage::SelectedRound(msg) => {
                self.selected.update(ctx, msg, &self.send_op_result)
            }
            RoundsViewMessage::ReQuery => {
                spawn_update_listener(ctx, RoundsViewMessage::ReQuery);
                self.scroll.requery(ctx);
                self.selected.try_requery_existing(ctx);
                false
            }
            RoundsViewMessage::BulkConfirmation => {
                let tracker = CLIENT.get().unwrap().update_tourn(
                    self.id,
                    TournOp::AdminOp(self.admin_id.clone().into(), AdminOp::ConfirmAllRounds),
                );
                let send_op_result = self.send_op_result.clone();
                spawn_local(async move {
                    console_log("Waiting for update to finish!");
                    send_op_result.emit(tracker.process().await.unwrap())
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = ctx.link().callback(|_| RoundsViewMessage::BulkConfirmation);
        html! {
            <div>
                <div class="row">
                    <div class="col">{ self.input.view() }</div>
                    <div class="col"><button onclick={cb}>{"Confirm all rounds"}</button></div>
                </div>
                <div class="d-flex flex-row my-4">
                    <div>
                        <div class="overflow-auto player-scroll-box px-4">
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
