use yew::prelude::*;

use squire_sdk::{client::state::ClientState, tournaments::TournamentId};
use squire_sdk::model::{players::PlayerStatus, rounds::RoundStatus};

use crate::CLIENT;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct OverviewProps {
    pub id: TournamentId,
}

pub struct TournOverview {
    pub id: TournamentId,
}

impl Component for TournOverview {
    type Message = ();
    type Properties = OverviewProps;

    fn create(ctx: &Context<Self>) -> Self {
        TournOverview { id: ctx.props().id }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |t| {
                let (activeRnds, certifiedRnds) = t.round_reg.rounds.values().fold((0, 0), |mut acc, r| {
                    match r.status {
                        RoundStatus::Open => acc.0 += 1,
                        RoundStatus::Certified => acc.1 += 1,
                        _ => {}
                    }
                    acc
                });

                let (registeredPlyrs, droppedPlyrs) = t.player_reg.players.values().fold((0, 0), |mut acc, p| {
                    match p.status {
                        PlayerStatus::Registered => acc.0 += 1,
                        PlayerStatus::Dropped => acc.1 += 1,
                        _ => {}
                    }
                    acc
                });

                html! {
                    <div class="m-lg-0 m-md-4 my-3">
                        <div class="p-5 bg-light rounded-3">
                            <div class="container-fluid p-md-5">
                                <h1 class="display-5 fw-bold">{ format!("Welcome to {}", t.name) }</h1>
                                <hr class="my-4"/>
                                <p>{ format!("Format : {}", t.format) }</p>
                                <p>{ format!("Status : {}", t.status) }</p>
                                <p>{ format!("Registered players : {registeredPlyrs}") }</p>

                                if droppedPlyrs > 0 {
                                    <p>{ format!("Dropped players : {droppedPlyrs}") }</p>
                                }

                                <p>{ format!("Active rounds : {activeRnds}") }</p>
                                <p>{ format!("Certified rounds : {certifiedRnds}") }</p>
                                <p>{ format!("Number of judges : {}", t.judges.len()) }</p>
                                <p>{ format!("Number of admins : {}", t.admins.len()) }</p>
                            </div>
                        </div>
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
