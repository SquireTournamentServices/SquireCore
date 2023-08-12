use std::{borrow::Cow, collections::HashMap};

use squire_sdk::model::{
    identifiers::AdminId,
    operations::{AdminOp, TournOp},
    pairings::Pairings,
    players::PlayerId,
    rounds::{Round, RoundId},
    tournament::{Tournament, TournamentId},
};
use wasm_bindgen_futures::spawn_local;
use yew::{prelude::*, virtual_dom::VNode};

use super::viewer_component::{
    InteractionResponse, TournViewerComponent, TournViewerComponentWrapper, WrapperMessage,
    WrapperState,
};
use crate::{
    utils::{generic_popout_window, generic_scroll_vnode, TextInput},
    CLIENT,
};

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
    PopoutActiveRounds(),
    /* Update tournament */
    GeneratePairings,
    PairingsToRounds,
    CreateSingleRound(),
    CreateSingleBye(),
    SingleRoundInput(usize, String),
    SingleByeInput(String),
}
#[derive(Debug, PartialEq, Clone)]
pub enum PairingsQueryMessage {
    /* Data ready */
    /*
    MatchSizeReady(u8),
    PlayerNamesReady(HashMap<PlayerId, String>),
    ActiveRoundsReady(Vec<ActiveRoundSummary>),
    */
    PairingsReady(PairingsWrapper),
    AllDataReady(),
}
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
    pub admin_id: AdminId,
    // names: Option<HashMap<PlayerId, String>>,
    // send_names: Callback<HashMap<PlayerId, String>>,
    pairings: Option<PairingsWrapper>,
    // active: Option<Vec<ActiveRoundSummary>>,
    // send_active: Callback<Vec<ActiveRoundSummary>>,
    // max_player_count: Option<u8>,
    // send_max_player_count: Callback<u8>,
    mode: PairingsViewMode,
    single_round_inputs: Vec<String>,
    single_bye_input: String,
    query_data: Option<PairingsQueryData>,
}

impl TournViewerComponent for PairingsView {
    type Properties = PairingsViewProps;
    type InteractionMessage = PairingsViewMessage;
    type QueryMessage = PairingsQueryMessage;

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self {
        // spawn_update_listener(ctx, PairingsViewMessage::PairingsReady() );
        /*
        let WrapperProps<PairingsViewProps> {
            id,
            admin_id,
            send_op_result,
            props
        } = ctx.props().props.clone();
        */
        let mut to_return = Self {
            id: ctx.props().t_id,
            admin_id: ctx.props().a_id,
            /*
            names: None,
            send_names: ctx.link().callback(WrapperMessage::Interaction(PairingsViewMessage::PlayerNamesReady) ),
            pairings: None,
            send_pairings: ctx.link().callback(PairingsViewMessage::PairingsReady),
            active: None,
            send_active: ctx.link().callback(PairingsViewMessage::ActiveRoundsReady),
            max_player_count: None,
            send_max_player_count: ctx.link().callback(PairingsViewMessage::MatchSizeReady),
            */
            mode: PairingsViewMode::CreatePairings,
            single_round_inputs: Vec::new(),
            single_bye_input: "".to_string(),
            query_data: None,
            pairings: None,
            // send_query_data: ctx.link().callback( WrapperMessage::QueryData(PairingsQueryMessage::AllDataReady) ),
        };
        to_return
    }

    fn v_view(
        &self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        state: &WrapperState,
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

    fn load_queried_data(&mut self, _msg: Self::QueryMessage, state: &WrapperState) -> bool {
        match _msg {
            /*
            PairingsQueryMessage::MatchSizeReady(msize) => {
                self.max_player_count = Some(msize);
                self.single_round_inputs = std::iter::repeat_with(String::new)
                    .take(msize.into())
                    .collect();
                true.into()
            }
            PairingsQueryMessage::PlayerNamesReady(pnhm) => {
                self.names = Some(pnhm);
                true.into()
            }
            PairingsQueryMessage::ActiveRoundsReady(v_ars) => {
                let mut v_ars = v_ars.clone();
                v_ars.sort_by_cached_key(|r| r.round_number);
                self.active = Some(v_ars);
                true.into()
            }
            */
            PairingsQueryMessage::AllDataReady(msize) => {
                todo!()
            }
            PairingsQueryMessage::PairingsReady(p) => {
                self.pairings = Some(p);
                true.into()
            }
        }
    }

    fn interaction(
        &mut self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        _msg: Self::InteractionMessage,
        state: &WrapperState,
    ) -> InteractionResponse<Self>{
        match _msg {
            PairingsViewMessage::ChangeMode(vm) => {
                if vm == PairingsViewMode::ActivePairings {
                    self.query_active_rounds(ctx);
                };
                self.mode = vm;
                true.into()
            }
            PairingsViewMessage::GeneratePairings => {
                let tracker = CLIENT.get().unwrap().query_tourn(self.id, |tourn| {
                    let pairings = tourn.create_pairings().unwrap_or_default();
                    PairingsWrapper { pairings }
                });
                let send_pairings = self.send_pairings.clone();
                spawn_local(async move { send_pairings.emit(tracker.process().await.unwrap()) });
                false.into()
            }
            PairingsViewMessage::PairingsToRounds => {
                let Some(pairings) = self.pairings.take() else {
                    return false.into()
                };
                let tracker = CLIENT.get().unwrap().update_tourn(
                    self.id,
                    TournOp::AdminOp(
                        self.admin_id.clone().into(),
                        AdminOp::PairRound(pairings.pairings),
                    ),
                );
                let send_op_result = state.send_op_result.clone();
                spawn_local(async move { send_op_result.emit(tracker.process().await.unwrap()) });
                true.into()
            }
            PairingsViewMessage::PopoutActiveRounds() => {
                if self.query_data.is_none() {
                    return false.into();
                }
                let scroll_strings = self.active.as_ref().unwrap().iter().map(|ars| {
                    //let player_list = ars.players.iter().map(|pn|{
                    //    format!("{}, ", pn)
                    //}).collect();
                    format!(
                        "Round #{}, Table #{} :: {}",
                        ars.round_number,
                        ars.table_number,
                        ars.players.join(", ")
                    )
                });
                let scroll_vnode = generic_scroll_vnode(90, scroll_strings);
                generic_popout_window(scroll_vnode.clone());
                false.into()
            }
            PairingsViewMessage::CreateSingleRound() => {
                if self.query_data.is_none() {
                    return false.into();
                };
                let player_ids: Vec<PlayerId> = self
                    .single_round_inputs
                    .iter()
                    .map(|plr_name| {
                        self.names
                            .as_ref()
                            .unwrap()
                            .iter()
                            .find_map(|(id, name)| (plr_name == name).then_some(*id))
                            .unwrap_or_default()
                    })
                    .collect();
                let tracker = CLIENT.get().unwrap().update_tourn(
                    self.id,
                    TournOp::AdminOp(
                        self.admin_id.clone().into(),
                        AdminOp::CreateRound(player_ids),
                    ),
                );
                let send_op_result = state.send_op_result.clone();
                spawn_local(async move { send_op_result.emit(tracker.process().await.unwrap()) });
                true.into()
            }
            PairingsViewMessage::CreateSingleBye() => {
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
                let tracker = CLIENT.get().unwrap().update_tourn(
                    self.id,
                    TournOp::AdminOp(self.admin_id.clone().into(), AdminOp::GiveBye(player_id)),
                );
                let send_op_result = state.send_op_result.clone();
                spawn_local(async move { send_op_result.emit(tracker.process().await.unwrap()) });
                true.into()
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

    fn query(&mut self, ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) {
        self.query_player_names(ctx);
        self.query_active_rounds(ctx);
        self.query_match_size(ctx);
    }
}

impl PairingsView {
    fn query_player_names(&mut self, _ctx: &Context<TournViewerComponentWrapper<Self>>) {
        let tracker = CLIENT.get().unwrap().query_tourn(self.id, |tourn| {
            let digest: HashMap<PlayerId, String> = tourn
                .player_reg
                .players
                .iter()
                .map(|(id, plyr)| (*id, plyr.name.clone()))
                .collect();
            digest
        });
        let send_names = self.send_names.clone();
        spawn_local(async move { send_names.emit(tracker.await.unwrap()) });
    }

    fn query_active_rounds(&mut self, _ctx: &Context<TournViewerComponentWrapper<Self>>) {
        let tracker = CLIENT.get().unwrap().query_tourn(self.id, |tourn| {
            tourn
                .get_active_rounds()
                .into_iter()
                .map(|r| ActiveRoundSummary::from_round(tourn, r))
                .collect()
        });
        let send_active = self.send_active.clone();
        spawn_local(async move { send_active.emit(tracker.await.unwrap()) });
    }

    fn query_match_size(&mut self, _ctx: &Context<TournViewerComponentWrapper<Self>>) {
        let tracker = CLIENT
            .get()
            .unwrap()
            .query_tourn(self.id, |tourn| tourn.pairing_sys.common.match_size);
        let send_match_size = self.send_max_player_count.clone();
        spawn_local(async move { send_match_size.emit(tracker.await.unwrap()) });
    }

    fn view_creation_menu(&self, ctx: &Context<TournViewerComponentWrapper<Self>>) -> Html {
        let cb_gen_pairings = ctx
            .link()
            .callback(move |_| PairingsViewMessage::GeneratePairings);
        let cb_gen_rounds = ctx
            .link()
            .callback(move |_| PairingsViewMessage::PairingsToRounds);
        html! {
            <div class="py-5">
                <button onclick={cb_gen_pairings} >{"Generate new pairings"}</button>
                <div class="overflow-auto py-3 pairings-scroll-box">
                    <ul class="force_left">{
                        if self.query_data.is_some()
                        {
                            self.query_data.as_ref().unwrap().pairings.clone().pairings.paired.into_iter().map( |p| {
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
        let cb_active_popout = ctx
            .link()
            .callback(move |_| PairingsViewMessage::PopoutActiveRounds());
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
            let max_players = self.query_data.unwrap().max_player_count.clone();
            for i in 0..max_players {
                let name_string = format!("player {}: ", i + 1);
                name_boxes.push(html!{
                    <>
                    <TextInput label = {Cow::from(name_string)} process = { ctx.link().callback(move |s| PairingsViewMessage::SingleRoundInput(i.into(), s)) } />
                    <br/>
                    </>
                })
            }
            let cb_single_round = ctx
                .link()
                .callback(move |_| PairingsViewMessage::CreateSingleRound());
            let cb_single_bye = ctx
                .link()
                .callback(move |_| PairingsViewMessage::CreateSingleBye());
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
                        <TextInput label = {Cow::from("Player to give bye :")} process = { ctx.link().callback(move |s| PairingsViewMessage::SingleByeInput(s)) } />
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
