use squire_sdk::model::{
    identifiers::TournamentId,
    players::{Player, PlayerId, PlayerStatus},
};
use yew::prelude::*;

use super::{input::PlayerFilterReport, PlayerView, PlayerViewMessage, SelectedPlayerMessage};
use crate::{CLIENT, tournament::viewer_component::TournViewerComponentWrapper};

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerScrollMessage {
    ScrollQueryReady(Vec<PlayerSummary>),
}

pub struct PlayerScroll {
    pub process: Callback<PlayerId>,
    players: Vec<PlayerSummary>,
}

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

impl PlayerScroll {
    pub fn new(ctx: &Context<TournViewerComponentWrapper<PlayerView>>, id: TournamentId) -> Self {
        fetch_player_summaries(ctx, id);
        Self {
            process: ctx.link().callback(SelectedPlayerMessage::PlayerSelected),
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
        /*
        let list = self
            .rounds
            .iter()
            .cloned()
            .filter_map(|r| {
                report.matches(&r).then(|| {
                    let cb = self.process.clone();
                    html! {
                        <tr onclick = { move |_| cb.emit(r.id) }>
                            <td>{ r.match_number }</td>
                            <td>{ r.table_number }</td>
                            <td>{ r.status }</td>
                        </tr>
                    }
                })
            })
            .collect::<Html>();
        html! {
            <table class="table">
                <thead>
                    <tr>
                        <th>{ "Round" }</th>
                        <th>{ "Table" }</th>
                        <th>{ "Status" }</th>
                    </tr>
                </thead>
                <tbody> { list } </tbody>
            </table>
        }
        */
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
