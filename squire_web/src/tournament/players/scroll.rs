use squire_sdk::model::{
    identifiers::TournamentId,
    players::{Player, PlayerId, PlayerStatus},
};
use yew::prelude::*;

use super::{input::PlayerFilterReport};

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerScrollMessage {
    ScrollQueryReady(Vec<PlayerSummary>),

}

pub struct PlayerScroll {
    pub process: Callback<PlayerId>,
    players: Vec<PlayerSummary>,
}

/*
pub fn fetch_player_summaries(ctx: &Context<TournViewerComponentWrapper<PlayerView>>, id: TournamentId) {
    ctx.link().send_future(async move {
        let mut data = CLIENT
            .get()
            .unwrap()
            .query_players(id, |plyrs| {
                plyrs
                    .players
                    .values()
                    .map(PlayerSummary::new)
                    .collect::<Vec<_>>()
            })
            .await
            .unwrap_or_default();
        data.sort_by_cached_key(|p| p.name.clone());
        data.sort_by_cached_key(|p| p.status);
        PlayerViewMessage::PlayerScroll(PlayerScrollMessage::ScrollQueryReady(data))
    })
}
*/

impl PlayerScroll {
    pub fn new(process: Callback<PlayerId>, _id: TournamentId) -> Self {
        //fetch_player_summaries(ctx, id);
        Self {
            process,
            players: Vec::default(),
        }
    }

    pub fn update(&mut self, msg: PlayerScrollMessage) -> bool {
        match msg {
            PlayerScrollMessage::ScrollQueryReady(data) => {
                let digest = self.players != data;
                self.players = data;
                digest
            }
        }
    }

    pub fn view(&self, report: PlayerFilterReport) -> Html {
        let mapper = |plyr: &PlayerSummary| {
            let cb = self.process.clone();
            let name = plyr.name.clone();
            let status = plyr.status.clone();
            let id = plyr.id;
            //html! { <li><a class="py-1 vert" onclick = { move |_| cb.emit(id) }>{ name }</a></li> }
            html! {
                <tr onclick = { move |_| cb.emit(id) }>
                    <td>{ name }</td>
                    <td>{ status }</td>
                </tr>
            }
        };
        let inner = self
            .players
            .iter()
            .filter_map(|p| report.matches(p).then(|| mapper(p)))
            .collect::<Html>();
        html! {
            <table class="table">
                <thead>
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Status" }</th>
                    </tr>
                </thead>
                <tbody> { inner } </tbody>
            </table>
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PlayerSummary {
    pub name: String,
    pub status: PlayerStatus,
    pub id: PlayerId,
}

impl PlayerSummary {
    pub fn new(plyr: &Player) -> Self {
        Self {
            name: plyr.name.clone(),
            status: plyr.status,
            id: plyr.id,
        }
    }
}
