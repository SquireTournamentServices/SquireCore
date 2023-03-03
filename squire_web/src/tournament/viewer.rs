use yew::{html, Component, Context, Html, Properties};

use squire_sdk::{client::state::ClientState, tournaments::TournamentId};

use crate::{client, CLIENT};

pub struct TournamentViewer {
    pub id: TournamentId,
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct TournProps {
    pub id: TournamentId,
}

impl Component for TournamentViewer {
    type Message = ();
    type Properties = TournProps;

    fn create(ctx: &Context<Self>) -> Self {
        let TournProps { id } = ctx.props();
        Self { id: *id }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let client = CLIENT.get().unwrap();
        client.state.query_tournament(&self.id, |t| {
            let tourn = t.tourn();
            html! {
                <div>
                    <h1 align="center">{ format!("Welcome to {}", tourn.name) }</h1>
                    <div>
                        <ul>
                        <li><a align="center" class="vert">{"Overview"}</a></li>
                        <li><a align="center" class="vert">{"Players"}</a></li>
                        <li><a align="center" class="vert">{"Rounds"}</a></li>
                        <li><a align="center" class="vert">{"Standings"}</a></li>
                        <li><a align="center" class="vert">{"Settings"}</a></li>
                        </ul>
                        <p width="95%" height="95%">{"Some text"}</p>
                    </div>
                </div>
            }
        }).unwrap_or_default()
    }
}
