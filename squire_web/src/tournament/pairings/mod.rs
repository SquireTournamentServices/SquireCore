use std::{borrow::Cow, collections::HashMap};

use squire_sdk::{
    model::{
        operations::{AdminOp, TournOp},
        pairings::Pairings,
        players::PlayerId,
        rounds::{Round, RoundId},
        tournament::{Tournament, TournamentId},
    },
    sync::TournamentManager,
};
use yew::{prelude::*, virtual_dom::VNode};

use super::viewer_component::{
    InteractionResponse, Op, TournViewerComponent, TournViewerComponentWrapper, WrapperMessage,
    WrapperState,
};
use crate::utils::{generic_popout_window, generic_scroll_vnode, TextInput};

#[derive(Debug, PartialEq, Clone)]
pub struct PairingsWrapper {
    pub pairings: Pairings,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ActiveRoundSummary {
    pub round_id: RoundId,
    pub round_number: u64,
    pub table_number: u64,
    pub players: Vec<String>,
}
impl ActiveRoundSummary {
    pub fn from_round(tourn: &Tournament, round_ref: &Round) -> Self {
        Self {
            round_id: round_ref.id,
            round_number: round_ref.match_number,
            table_number: round_ref.table_number,
            players: round_ref
                .players
                .iter()
                .map(|pid| tourn.get_player_by_id(pid).unwrap().name.clone())
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct PairingsViewProps {}

#[derive(Debug, PartialEq, Clone)]
pub enum PairingsViewMessage {
    /* Interaction */
    ChangeMode(PairingsViewMode),
    PopoutActiveRounds,
    /* Update tournament */
    GeneratePairings,
    PairingsToRounds,
    CreateSingleRound,
    CreateSingleBye,
    SingleRoundInput(usize, String),
    SingleByeInput(String),
}
#[derive(Debug, PartialEq, Clone)]
pub enum PairingsQueryMessage {
    PairingsReady(PairingsWrapper),
    AllDataReady(PairingsQueryData),
}
#[derive(Debug, PartialEq, Clone)]
pub struct PairingsQueryData {
    names: HashMap<PlayerId, String>,
    active: Vec<ActiveRoundSummary>,
    max_player_count: u8,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PairingsViewMode {
    CreatePairings,
    ActivePairings,
    CreateSingleMatches,
}

pub struct PairingsView {
    pub id: TournamentId,
    pairings: Option<PairingsWrapper>,
    mode: PairingsViewMode,
    single_round_inputs: Vec<String>,
    single_bye_input: String,
    query_data: Option<PairingsQueryData>,
}

impl TournViewerComponent for PairingsView {
    type Properties = PairingsViewProps;
    type InteractionMessage = PairingsViewMessage;
    type QueryMessage = PairingsQueryMessage;

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, _state: &WrapperState) -> Self {
        let to_return = Self {
            id: ctx.props().t_id,
            mode: PairingsViewMode::CreatePairings,
            single_round_inputs: Vec::new(),
            single_bye_input: "".to_string(),
            query_data: None,
            pairings: None,
        };
        to_return
    }

    fn v_view(
        &self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> Html {
        let make_callback = |mode| {
            ctx.link().callback(move |_| {
                WrapperMessage::Interaction(PairingsViewMessage::ChangeMode(mode))
            })
        };
        let make_button =
            |name, mode| html! { <button onclick = { make_callback(mode) }>{name}</button> };
        html! {
            <div>
                <h1>{ "Pairings" }</h1>
                <div>
                    <>{ make_button("Create Pairings", PairingsViewMode::CreatePairings) }</>
                    <>{ make_button("Active Pairings", PairingsViewMode::ActivePairings) }</>
                    <>{ make_button("Create single matches", PairingsViewMode::CreateSingleMatches) }</>
                </div>
                <div>{
                    match self.mode {
                        PairingsViewMode::CreatePairings => {
                            self.view_creation_menu(ctx)
                        }
                        PairingsViewMode::ActivePairings => {
                            self.view_active_menu(ctx)
                        }
                        PairingsViewMode::CreateSingleMatches => {
                            self.view_single_menu(ctx)
                        }
                    }
                }</div>
            </div>
        }
    }

    fn load_queried_data(&mut self, msg: Self::QueryMessage, _state: &WrapperState) -> bool {
        match msg {
            PairingsQueryMessage::AllDataReady(data) => {
                self.query_data = Some(data);
                true
            }
            PairingsQueryMessage::PairingsReady(p) => {
                self.pairings = Some(p);
                true
            }
        }
    }

    fn interaction(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _msg: Self::InteractionMessage,
        state: &WrapperState,
    ) -> InteractionResponse<Self> {
        match _msg {
            PairingsViewMessage::ChangeMode(vm) => {
                self.mode = vm;
                true.into()
            }
            PairingsViewMessage::GeneratePairings => {
                let q_func = |tourn: &TournamentManager| {
                    let pairings = tourn.create_pairings().unwrap_or_default();
                    Self::QueryMessage::PairingsReady(PairingsWrapper { pairings })
                };
                InteractionResponse::FetchData(Box::new(q_func))
            }
            PairingsViewMessage::PairingsToRounds => {
                let Some(pairings) = self.pairings.take() else {
                    return false.into()
                };
                state
                    .get_user_id()
                    .map(|user_id| {
                        let ops = vec![TournOp::AdminOp(
                            user_id.convert(),
                            AdminOp::PairRound(pairings.pairings),
                        )];
                        InteractionResponse::Update(ops)
                    })
                    .unwrap_or_default()
            }
            PairingsViewMessage::PopoutActiveRounds => {
                if self.query_data.is_none() {
                    return false.into();
                }
                let scroll_strings = self.query_data.as_ref().unwrap().active.iter().map(|ars| {
                    format!(
                        "Round #{}, Table #{} :: {}",
                        ars.round_number,
                        ars.table_number,
                        ars.players.join(", ")
                    )
                });
                let scroll_vnode = generic_scroll_vnode(90, scroll_strings);
                generic_popout_window(scroll_vnode);
                false.into()
            }
            PairingsViewMessage::CreateSingleRound => {
                if self.query_data.is_none() {
                    return false.into();
                };
                let player_ids: Vec<PlayerId> = self
                    .single_round_inputs
                    .iter()
                    .map(|plr_name| {
                        self.query_data
                            .as_ref()
                            .unwrap()
                            .names
                            .iter()
                            .find_map(|(id, name)| (plr_name == name).then_some(*id))
                            .unwrap_or_default()
                    })
                    .collect();
                let mut ops = Vec::new();
                ops.push(Op::Admin(AdminOp::CreateRound(player_ids)));
                state.op_response(ops)
            }
            PairingsViewMessage::CreateSingleBye => {
                if self.query_data.is_none() {
                    return false.into();
                };
                let player_id: PlayerId = self
                    .query_data
                    .as_ref()
                    .unwrap()
                    .names
                    .iter()
                    .find_map(|(id, name)| (self.single_bye_input == *name).then_some(*id))
                    .unwrap_or_default();
                let mut ops = Vec::new();
                ops.push(Op::Admin(AdminOp::GiveBye(player_id)));
                state.op_response(ops)
            }
            PairingsViewMessage::SingleRoundInput(vec_index, text) => {
                let Some(name) = self.single_round_inputs.get_mut(vec_index) else {
                    return false.into();
                };
                *name = text;
                false.into()
            }
            PairingsViewMessage::SingleByeInput(text) => {
                self.single_bye_input = text;
                false.into()
            }
        }
    }

    fn query(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> Box<dyn 'static + Send + FnOnce(&TournamentManager) -> Self::QueryMessage> // <- we probably want to alias over this at some point
    {
        let q_func = |tourn: &TournamentManager| {
            let names: HashMap<PlayerId, String> = tourn
                .player_reg
                .players
                .iter()
                .map(|(id, plyr)| (*id, plyr.name.clone()))
                .collect();
            let active: Vec<ActiveRoundSummary> = tourn
                .get_active_rounds()
                .into_iter()
                .map(|r| ActiveRoundSummary::from_round(tourn, r))
                .collect();
            let max_player_count: u8 = tourn.pairing_sys.common.match_size;
            Self::QueryMessage::AllDataReady(PairingsQueryData {
                names,
                active,
                max_player_count,
            })
        };
        Box::new(q_func)
    }
}

impl PairingsView {
    fn view_creation_menu(&self, ctx: &Context<TournViewerComponentWrapper<Self>>) -> Html {
        let cb_gen_pairings = ctx
            .link()
            .callback(move |_| WrapperMessage::Interaction(PairingsViewMessage::GeneratePairings));
        let cb_gen_rounds = ctx
            .link()
            .callback(move |_| WrapperMessage::Interaction(PairingsViewMessage::PairingsToRounds));
        html! {
            <div class="py-5">
                <button onclick={cb_gen_pairings} >{"Generate new pairings"}</button>
                <div class="overflow-auto py-3 pairings-scroll-box">
                    <ul class="force_left">{
                        if self.query_data.is_some() && self.pairings.is_some()
                        {
                            self.pairings.as_ref().unwrap().clone().pairings.paired.into_iter().map( |p| {
                                html!{
                                    <li>{
                                        p.into_iter().map(|pid|{
                                            html!{<><>{self.query_data.as_ref().unwrap().names.get(&pid)}</><>{", "}</></>}
                                        })
                                        .collect::<Html>()
                                    }</li>
                                }
                            })
                            .collect::<Html>()
                        }
                        else
                        {
                            html!{<li>{"..."}</li>}
                        }
                    }</ul>
                </div>
                <button onclick={cb_gen_rounds} disabled={self.query_data.is_none()}>{"Turn pairings into live rounds"}</button>
            </div>
        }
    }

    fn view_active_menu(&self, ctx: &Context<TournViewerComponentWrapper<Self>>) -> Html {
        let cb_active_popout = ctx.link().callback(move |_| {
            WrapperMessage::Interaction(PairingsViewMessage::PopoutActiveRounds)
        });
        html! {
            <div class="py-5">
                <div class="overflow-auto py-3 pairings-scroll-box">
                    <ul class="force_left">{
                        if self.query_data.is_some()
                        {
                            self.query_data.as_ref().unwrap().active.clone().into_iter().map( |ars| {
                                html!{
                                    <li>
                                        <>{ format!("Round #{}, Table #{} :: ", ars.round_number, ars.table_number) }</>
                                        <>{
                                            ars.players.iter().map(|pn| {
                                                html!{<>  { format!("{}, ", pn) }  </>}
                                            })
                                            .collect::<Html>()
                                        }</>
                                    </li>
                                }
                            })
                            .collect::<Html>()
                        }
                        else
                        {
                            html!{<li>{"..."}</li>}
                        }
                    }</ul>
                </div>
                <button onclick={cb_active_popout} >{ "Pop-out Scrolling Display" }</button>
            </div>
        }
    }

    fn view_single_menu(&self, ctx: &Context<TournViewerComponentWrapper<Self>>) -> Html {
        if self.query_data.is_some() {
            let mut name_boxes: Vec<VNode> = Vec::new();
            let max_players = self.query_data.as_ref().unwrap().max_player_count;
            for i in 0..max_players {
                let name_string = format!("player {}: ", i + 1);
                name_boxes.push(html!{
                    <>
                    <TextInput label = {Cow::from(name_string)} process = { ctx.link().callback(move |s| WrapperMessage::Interaction(PairingsViewMessage::SingleRoundInput(i.into(), s))) } />
                    <br/>
                    </>
                })
            }
            let cb_single_round = ctx.link().callback(move |_| {
                WrapperMessage::Interaction(PairingsViewMessage::CreateSingleRound)
            });
            let cb_single_bye = ctx.link().callback(move |_| {
                WrapperMessage::Interaction(PairingsViewMessage::CreateSingleBye)
            });
            html! {
                <div class="py-5">
                    <h2>{ "Create single rounds: " }</h2>
                    <div class="py-1">{
                        name_boxes
                    }</div>
                    <button onclick={cb_single_round}>{ "Create round" }</button>
                    <hr />
                    <h2>{ "Create a bye: " }</h2>
                    <div class="py-2">
                        <>
                        <TextInput label = {Cow::from("Player to give bye :")} process = { ctx.link().callback(move |s| WrapperMessage::Interaction(PairingsViewMessage::SingleByeInput(s))) } />
                        <br/>
                        </>
                    </div>
                    <button onclick={cb_single_bye} >{ "Create bye" }</button>
                </div>
            }
        } else {
            html! {
                <>{"..."}</>
            }
        }
    }
}
