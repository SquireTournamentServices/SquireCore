use yew::prelude::*;

use squire_sdk::{client::state::ClientState, tournaments::TournamentId};

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
                let activeRounds = t.round_reg.rounds.values().filter(|r| r.is_active()).count();
                let certifiedRounds = t.round_reg.rounds.values().filter(|r| r.is_certified()).count();
                let registeredPlayers = t.player_reg.players.values().filter(|p| p.can_play()).count();
                let droppedPlayers = t.player_reg.players.values().filter(|p| !p.can_play()).count();

                html! {
                    <div class="m-lg-0 m-md-4 m-0">
                        <div class="p-5 bg-light rounded-3">
                            <div class="container-fluid p-md-5">
                                <h1 class="display-5 fw-bold">{ format!("Welcome to {}", t.name) }</h1>
                                <hr class="my-4"/>
                                <p>{ format!("Format : {}", t.format) }</p>
                                <p>{ format!("Status : {}", t.status) }</p>
                                <p>{ format!("Registered players : {}", registeredPlayers) }</p>

                                if droppedPlayers > 0 {
                                    <p>{ format!("Dropped players : {}", droppedPlayers) }</p>
                                }

                                <p>{ format!("Active rounds : {}", activeRounds) }</p>
                                <p>{ format!("Certified rounds : {}", certifiedRounds) }</p>
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
