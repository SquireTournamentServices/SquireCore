use squire_sdk::{
    model::{identifiers::PlayerIdentifier, rounds::RoundId},
    model::{identifiers::RoundIdentifier, rounds::RoundStatus},
    players::PlayerId,
    tournaments::{Tournament, TournamentId, TournamentManager},
};

use yew::prelude::*;

use crate::{utils::{TextInput, input, console_log}, CLIENT};

use super::{rounds::SelectedRound, creator};


#[derive(Debug, PartialEq, Properties)]
pub struct PairingsViewProps {
    pub id: TournamentId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PairingsViewMessage {
    ChangeMode(PairingsViewMode)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PairingsViewMode {
    CreatePairings,
    ActivePairings,
    CreateSingleMatches,
}

pub struct PairingsView {
    pub id: TournamentId,
    mode: PairingsViewMode
}

impl Component for PairingsView {
    type Message = PairingsViewMessage;
    type Properties = PairingsViewProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().id;
        Self {
            id,
            mode : PairingsViewMode::CreatePairings
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PairingsViewMessage::ChangeMode(vm) => {
                self.mode = vm;
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
                            "Create pairings"
                        }
                        PairingsViewMode::ActivePairings => {
                            "Active pairings"
                        }
                        PairingsViewMode::CreateSingleMatches => {
                            "Create single matches"
                        }
                    }
                }</div>
            </div>
        }
    }
}
