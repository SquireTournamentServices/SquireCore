use std::collections::HashMap;

use js_sys::Math::round;
use squire_sdk::{
    model::{identifiers::PlayerIdentifier, rounds::RoundId},
    model::{identifiers::{RoundIdentifier, AdminId}, rounds::RoundStatus, pairings::Pairings, operations::AdminOp},
    players::{PlayerId, Player, Round},
    tournaments::{Tournament, TournamentId, TournamentManager, OpResult, TournOp},
};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{utils::{TextInput, input, console_log}, CLIENT};

use super::{rounds::SelectedRound, creator, spawn_update_listener};


#[derive(Debug, PartialEq, Clone)]
pub struct PairingsWrapper {
    pub pairings: Pairings,
    pub names: HashMap<PlayerId, String>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ActiveRoundSummary {
    pub round_id: RoundId,
    pub table_number: u64,
    pub players: Vec<String>,
}
impl ActiveRoundSummary {
    pub fn from_round(tourn : &Tournament, round_ref : &Round) -> Self {
        Self {
            round_id : round_ref.id,
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
    ChangeMode(PairingsViewMode),
    GeneratePairings,
    PairingsToRounds,
    PairingsReady(PairingsWrapper),
    QueryActiveRounds,
    ActiveRoundsReady(Vec<ActiveRoundSummary>),
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
    mode: PairingsViewMode,
    pairings: Option<PairingsWrapper>,
    send_pairings: Callback<PairingsWrapper>,
    active: Option<Vec<ActiveRoundSummary>>,
    send_active: Callback<Vec<ActiveRoundSummary>>,
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
        Self {
            id,
            admin_id,
            mode : PairingsViewMode::CreatePairings,
            pairings : None,
            send_pairings : ctx.link().callback(PairingsViewMessage::PairingsReady ),
            active : None,
            send_active : ctx.link().callback(PairingsViewMessage::ActiveRoundsReady ),
            send_op_result,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
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
                        let names = pairings
                            .paired
                            .iter()
                            .map(|p| p.iter())
                            .flatten()
                            .chain(pairings.rejected.iter()) // Iterator over all the player id in the pairings
                            .filter_map(|id| {
                                tourn
                                    .get_player_by_id(id)
                                    .map(|plyr| (*id, plyr.name.clone()))
                                    .ok()
                            })
                            .collect();
                        PairingsWrapper { pairings, names }
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
                self.active = Some(v_ars);
                true
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
                        if (self.pairings.is_some())
                        {
                            self.pairings.as_ref().unwrap().clone().pairings.paired.into_iter().map( |p| {
                                html!{
                                    <li>{
                                        p.into_iter().map(|pid|{
                                            html!{<><>{self.pairings.as_ref().unwrap().names.get(&pid)}</><>{", "}</></>}
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
                <button onclick={cb_gen_rounds} >{"Turn pairings into live rounds"}</button>
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
                                    <li>{
                                        "It's here!"
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
            </div>
        }
    }

    fn view_single_menu(&self, ctx: &Context<Self>) -> Html {
        html!{
            <>
            </>
        }
    }

    fn view_pairing(&self) -> Html {
        html!{
            <>
            </>
        }
    }

}
