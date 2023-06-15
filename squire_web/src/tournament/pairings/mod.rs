use std::{collections::HashMap, hash::Hash, borrow::Cow};

use futures::future::OrElse;
use js_sys::Math::round;
use squire_sdk::{
    model::{identifiers::PlayerIdentifier, rounds::RoundId},
    model::{identifiers::{RoundIdentifier, AdminId}, rounds::RoundStatus, pairings::Pairings, operations::AdminOp},
    players::{PlayerId, Player, Round},
    tournaments::{Tournament, TournamentId, TournamentManager, OpResult, TournOp},
};

use wasm_bindgen_futures::spawn_local;
use yew::{prelude::*, virtual_dom::VNode};

use crate::{utils::{TextInput, input, console_log}, CLIENT};

use super::{rounds::SelectedRound, creator, spawn_update_listener};


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
    pub fn from_round(tourn : &Tournament, round_ref : &Round) -> Self {
        Self {
            round_id : round_ref.id,
            round_number : round_ref.match_number,
            table_number : round_ref.table_number,
            players : round_ref.players.iter().map(|pid|{
                tourn.get_player_by_id(pid).unwrap().name.clone()
            })
            .collect()
        }
    }
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct PairingsViewProps {
    pub id: TournamentId,
    pub admin_id: AdminId,
    pub send_op_result: Callback<OpResult>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PairingsViewMessage {
    QueryPlayerNames,
    PlayerNamesReady(HashMap<PlayerId, String>),
    ChangeMode(PairingsViewMode),
    GeneratePairings,
    PairingsToRounds,
    PairingsReady(PairingsWrapper),
    QueryActiveRounds,
    ActiveRoundsReady(Vec<ActiveRoundSummary>),
    QueryMatchSize,
    MatchSizeReady(u8),
    CreateSingleRound(),
    CreateSingleBye(String),
    SingleRoundInput(usize, String),
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
    names: Option<HashMap<PlayerId, String>>,
    send_names: Callback<HashMap<PlayerId, String>>,
    mode: PairingsViewMode,
    pairings: Option<PairingsWrapper>,
    send_pairings: Callback<PairingsWrapper>,
    active: Option<Vec<ActiveRoundSummary>>,
    send_active: Callback<Vec<ActiveRoundSummary>>,
    max_player_count: Option<u8>,
    send_max_player_count: Callback<u8>,
    single_round_inputs: Vec<String>,
    pub send_op_result: Callback<OpResult>,
}

impl Component for PairingsView {
    type Message = PairingsViewMessage;
    type Properties = PairingsViewProps;

    fn create(ctx: &Context<Self>) -> Self {
        // spawn_update_listener(ctx, PairingsViewMessage::PairingsReady() );
        let PairingsViewProps {
            id,
            admin_id,
            send_op_result,
        } = ctx.props().clone();
        let mut to_return = Self {
            id,
            admin_id,
            names: None,
            send_names: ctx.link().callback(PairingsViewMessage::PlayerNamesReady ),
            mode : PairingsViewMode::CreatePairings,
            pairings : None,
            send_pairings : ctx.link().callback(PairingsViewMessage::PairingsReady ),
            active : None,
            send_active : ctx.link().callback(PairingsViewMessage::ActiveRoundsReady ),
            max_player_count: None,
            send_max_player_count: ctx.link().callback(PairingsViewMessage::MatchSizeReady),
            single_round_inputs: Vec::new(),
            send_op_result,
        };
        to_return.query_player_names(ctx);
        to_return.query_match_size(ctx);
        to_return
    }

    // tourn.pairing_sys.common.match_size
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PairingsViewMessage::QueryPlayerNames => {
                self.query_player_names(ctx);
                false
            }
            PairingsViewMessage::PlayerNamesReady(pnhm) => {
                self.names = Some(pnhm);
                true
            }
            PairingsViewMessage::ChangeMode(vm) => {
                if (vm == PairingsViewMode::ActivePairings) {
                    self.query_active_rounds(ctx);
                };
                self.mode = vm;
                true
            }
            PairingsViewMessage::GeneratePairings => {
                let tracker =  CLIENT
                .get()
                .unwrap()
                .query_tourn(self.id, 
                    |tourn| {
                        let pairings = tourn.create_pairings().unwrap_or_default();
                        PairingsWrapper { pairings }
                    }
                );
                let send_pairings = self.send_pairings.clone();
                spawn_local(async move {
                    console_log("Waiting for update to finish!");
                    send_pairings.emit(tracker.process().await.unwrap()) 
                });
                false
            }
            PairingsViewMessage::PairingsToRounds => {
                let Some(pairings) = self.pairings.take() else { return false };
                let tracker =  CLIENT
                .get()
                .unwrap()
                .update_tourn(self.id, 
                    TournOp::AdminOp(self.admin_id.clone().into(), AdminOp::PairRound(pairings.pairings))
                );
                let send_op_result = self.send_op_result.clone();
                spawn_local(async move {
                    console_log("Waiting for update to finish!");
                    send_op_result.emit(tracker.process().await.unwrap()) 
                });
                true
            }
            PairingsViewMessage::PairingsReady(p) => {
                self.pairings = Some(p);
                true
            }
            PairingsViewMessage::QueryActiveRounds => {
                self.query_active_rounds(ctx);
                false
            }
            PairingsViewMessage::ActiveRoundsReady(v_ars) => {
                let mut v_ars = v_ars.clone();
                v_ars.sort_by_cached_key(|r| r.round_number);
                self.active = Some(v_ars);
                true
            }
            PairingsViewMessage::QueryMatchSize => {
                self.query_match_size(ctx);
                false
            }
            PairingsViewMessage::MatchSizeReady(msize) => {
                self.max_player_count = Some(msize);
                self.single_round_inputs = std::iter::repeat_with(String::new).take(msize.into()).collect();
                true
            }
            PairingsViewMessage::CreateSingleRound() => {
                if (self.names.is_none()) { return false};
                let player_ids : Vec<PlayerId> = self.single_round_inputs.iter().map( |plr_name| {
                    self.names.as_ref().unwrap().iter().find_map(|(id, name)|{
                        (plr_name == name).then_some(*id)
                    }).unwrap_or_default()
                })
                .collect();
                let tracker =  CLIENT
                .get()
                .unwrap()
                .update_tourn(self.id, 
                    TournOp::AdminOp(self.admin_id.clone().into(), AdminOp::CreateRound(player_ids))
                );
                let send_op_result = self.send_op_result.clone();
                spawn_local(async move {
                    console_log("Waiting for update to finish!");
                    send_op_result.emit(tracker.process().await.unwrap()) 
                });
                true
            }
            PairingsViewMessage::CreateSingleBye(player) => {
                false
            }
            PairingsViewMessage::SingleRoundInput(vec_index, text) => {
                let Some(name) = self.single_round_inputs.get_mut(vec_index) else { return false };
                *name = text;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let make_callback = |mode| {
            ctx.link()
                .callback(move |_| PairingsViewMessage::ChangeMode(mode))
        };
        let make_button = |name, mode| html! { <button onclick = { make_callback(mode) }>{name}</button> };
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
}

impl PairingsView {

    fn query_player_names(&mut self, ctx: &Context<Self>) {
        let tracker =  CLIENT
        .get()
        .unwrap()
        .query_tourn(self.id, 
            |tourn| {
                let digest: HashMap<PlayerId, String> = tourn.player_reg.players.iter().map(|(id, plyr)| (*id, plyr.name.clone())).collect();
                digest
            }
        );
        let send_names = self.send_names.clone();
        spawn_local(async move {
            console_log("Waiting for update to finish!");
            send_names.emit(tracker.process().await.unwrap()) 
        });
    }

    fn query_active_rounds(&mut self, ctx: &Context<Self>) {
        let tracker =  CLIENT
        .get()
        .unwrap()
        .query_tourn(self.id, 
            |tourn| {
                tourn.get_active_rounds().into_iter().map( |r| {
                    ActiveRoundSummary::from_round(tourn, r)
                })
                .collect()
            }
        );
        let send_active = self.send_active.clone();
        spawn_local(async move {
            console_log("Waiting for update to finish!");
            send_active.emit(tracker.process().await.unwrap()) 
        });
    }

    fn query_match_size(&mut self, ctx: &Context<Self>) {
        let tracker =  CLIENT
        .get()
        .unwrap()
        .query_tourn(self.id, 
            |tourn| {
                tourn.pairing_sys.common.match_size
            }
        );
        let send_match_size = self.send_max_player_count.clone();
        spawn_local(async move {
            console_log("Waiting for update to finish!");
            send_match_size.emit(tracker.process().await.unwrap())
        });
    }

    fn view_creation_menu(&self, ctx: &Context<Self>) -> Html {
        let cb_gen_pairings = ctx.link()
        .callback(move |_| PairingsViewMessage::GeneratePairings);
        let cb_gen_rounds = ctx.link()
        .callback(move |_| PairingsViewMessage::PairingsToRounds);
        html!{
            <div class="py-5">
                <button onclick={cb_gen_pairings} >{"Generate new pairings"}</button>
                <div class="overflow-auto py-3 pairings-scroll-box">
                    <ul class="force_left">{
                        if (self.pairings.is_some() && self.names.is_some())
                        {
                            self.pairings.as_ref().unwrap().clone().pairings.paired.into_iter().map( |p| {
                                html!{
                                    <li>{
                                        p.into_iter().map(|pid|{
                                            html!{<><>{self.names.as_ref().unwrap().get(&pid)}</><>{", "}</></>}
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
                <button onclick={cb_gen_rounds} disabled={self.pairings.is_none()}>{"Turn pairings into live rounds"}</button>
            </div>
        }
    }

    fn view_active_menu(&self, ctx: &Context<Self>) -> Html {
        html!{
            <div class="py-5">
                <div class="overflow-auto py-3 pairings-scroll-box">
                    <ul class="force_left">{
                        if (self.active.is_some())
                        {
                            self.active.as_ref().unwrap().clone().into_iter().map( |ars| {
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
            </div>
        }
    }

    fn view_single_menu(&self, ctx: &Context<Self>) -> Html {
        if (self.max_player_count.is_some() && self.names.is_some())
        {
            let mut name_boxes : Vec<VNode> = Vec::new();
            let max_players = self.max_player_count.unwrap().clone();
            for i in 0..max_players {
                let name_string = format!("player {}: ", i+1);
                name_boxes.push(html!{
                    <>
                    <TextInput label = {Cow::from(name_string)} process = { ctx.link().callback(move |s| PairingsViewMessage::SingleRoundInput(i.into(), s)) }/>
                    <br/>
                    </>
                })
            };
            let cb_single_round = ctx.link()
            .callback(move |_| PairingsViewMessage::CreateSingleRound() );
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
                        <label for="bye_name">{ "Player to give bye" }</label>
                        <input type="text" id="bye_name" name="bye_name" />
                        <br/><br/>
                    </div>
                    <button>{ "Create bye" }</button>
                </div>
            }
        }
        else
        {
            html! {
                <>{"..."}</>
            }
        }
    }

}
