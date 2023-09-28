use squire_sdk::{model::{
    identifiers::{AdminId, TournamentId}, operations::{TournOp, AdminOp},
}, sync::TournamentManager};
use yew::prelude::*;

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

use super::{viewer_component::{TournViewerComponent, WrapperMessage, InteractionResponse}, model::RoundProfile};

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct RoundsFilterProps {}

#[derive(Debug, PartialEq, Clone)]
pub enum RoundsViewMessage {
    FilterInput(RoundFilterInputMessage),
    RoundScroll(RoundScrollMessage),
    SelectedRound(SelectedRoundMessage),
    //ReQuery,
    BulkConfirmation,
}

pub enum RoundsViewQueryMessage {
    AllDataReady(RoundsViewQueryData),
    SelectedRoundReady(Option<RoundProfile>),
}

pub struct RoundsViewQueryData {
    rounds: Vec<RoundSummary>
}

pub struct RoundsView {
    pub id: TournamentId,
    pub admin_id: AdminId,
    input: RoundFilterInput,
    scroll: RoundScroll,
    selected: SelectedRound,
}

impl TournViewerComponent for RoundsView {
    type Properties = RoundsFilterProps;
    type InteractionMessage = RoundsViewMessage;
    type QueryMessage = RoundsViewQueryMessage;

    /*
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
                spawn_local(async move { send_op_result.emit(tracker.await.unwrap()) });
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
    */

    fn v_create(ctx: &Context<super::viewer_component::TournViewerComponentWrapper<Self>>, state: &super::viewer_component::WrapperState) -> Self {
        Self {
            id: state.t_id,
            input: RoundFilterInput::new(ctx.link().callback(|input| WrapperMessage::Interaction(RoundsViewMessage::FilterInput(input))) ),
            scroll: RoundScroll::new(ctx, state.t_id),
            admin_id: state.a_id,
            selected: SelectedRound::new(ctx, state.t_id, state.a_id),
        }
    }

    fn interaction(
            &mut self,
            ctx: &Context<super::viewer_component::TournViewerComponentWrapper<Self>>,
            msg: Self::InteractionMessage,
            _state: &super::viewer_component::WrapperState,
        ) -> super::viewer_component::InteractionResponse<Self> {
            match msg {
                RoundsViewMessage::FilterInput(msg) => self.input.update(msg).into(),
                RoundsViewMessage::RoundScroll(msg) => self.scroll.update(msg).into(),
                RoundsViewMessage::SelectedRound(msg) => {
                    self.selected.update(ctx, msg).into()
                }
                /*
                RoundsViewMessage::ReQuery => {
                    spawn_update_listener(ctx, RoundsViewMessage::ReQuery);
                    self.scroll.requery(ctx);
                    self.selected.try_requery_existing(ctx);
                    false
                }
                */
                RoundsViewMessage::BulkConfirmation => {
                    /*
                    let tracker = CLIENT.get().unwrap().update_tourn(
                        self.id,
                        TournOp::AdminOp(self.admin_id.clone().into(), AdminOp::ConfirmAllRounds),
                    );
                    let send_op_result = self.send_op_result.clone();
                    spawn_local(async move { send_op_result.emit(tracker.await.unwrap()) });
                    false
                    */
                    let ops = vec![TournOp::AdminOp(
                        self.admin_id.clone().into(),
                        AdminOp::ConfirmAllRounds,
                    )];
                    InteractionResponse::Update(ops)
                }
            }
    }

    fn query(
        &mut self,
        _ctx: &Context<super::viewer_component::TournViewerComponentWrapper<Self>>,
        _state: &super::viewer_component::WrapperState,
    ) -> super::viewer_component::TournQuery<Self::QueryMessage> {
        let q_func = |tourn: &TournamentManager| {
            let rounds : Vec<RoundSummary> = tourn
            .round_reg
            .rounds
            .values()
            .map(RoundSummary::new)
            .collect();
            Self::QueryMessage::AllDataReady(RoundsViewQueryData {
                rounds
            })
        };
        Box::new(q_func)
    }

    fn load_queried_data(&mut self, msg: Self::QueryMessage, _state: &super::viewer_component::WrapperState) -> bool {
        match msg {
            RoundsViewQueryMessage::AllDataReady(data) => {
                self.scroll.update(RoundScrollMessage::ScrollQueryReady(data.rounds))
            }
            RoundsViewQueryMessage::SelectedRoundReady(rnd) => {
                self.selected.round_query_ready(rnd);
                true
            }
        }
    }

    fn v_view(
        &self,
        ctx: &Context<super::viewer_component::TournViewerComponentWrapper<Self>>,
        _state: &super::viewer_component::WrapperState,
    ) -> yew::Html {
        let cb = ctx.link().callback(|_| WrapperMessage::Interaction(RoundsViewMessage::BulkConfirmation) );
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
