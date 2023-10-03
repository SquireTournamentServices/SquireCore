use squire_sdk::model::{identifiers::TournamentId, tournament::Tournament, r64};
use yew::{prelude::*, virtual_dom::VNode};

use crate::{
    utils::{generic_popout_window, generic_scroll_vnode, rational_to_float},
    CLIENT,
};

#[derive(Properties, PartialEq)]
struct StandingsPopoutProps {
    pub display_vnode: VNode,
}
#[function_component]
fn StandingsPopout(props: &StandingsPopoutProps) -> Html {
    props.display_vnode.clone()
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct StandingsProps {
    pub id: TournamentId,
}
#[derive(Debug, PartialEq, Clone)]
pub enum StandingsMessage {
    StandingsQueryReady(Option<StandingsProfile>), // Optional for the same reasons
    SpawnPopout(i32),
    SpawnPopoutStatic()
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StandingsProfile {
    standings: Vec<(usize, String, r64, r64, r64)>,
}

pub struct StandingsView {
    pub id: TournamentId,
    pub scroll_vnode: Option<VNode>,
    pub process: Callback<i32>,
    standings: StandingsProfile,
}

pub fn fetch_standings_profile(ctx: &Context<StandingsView>, id: TournamentId) {
    ctx.link().send_future(async move {
        let data = CLIENT
            .get()
            .unwrap()
            .query_tourn(id, |t| StandingsProfile::new(t.tourn()))
            .process()
            .await;
        StandingsMessage::StandingsQueryReady(data)
    })
}

impl Component for StandingsView {
    type Message = StandingsMessage;
    type Properties = StandingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let to_return = StandingsView {
            id: ctx.props().id,
            scroll_vnode: None,
            process: ctx.link().callback(StandingsMessage::SpawnPopout),
            standings: Default::default(),
        };
        fetch_standings_profile(ctx, to_return.id);
        to_return
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            StandingsMessage::SpawnPopout(_num) => {
                let scroll_strings = self
                    .standings
                    .standings
                    .iter()
                    .map(|(i, s, _points, _mwp, _opp_mwp)| format!("{i} : {s}"));
                self.scroll_vnode = Some(generic_scroll_vnode(120, scroll_strings));
                generic_popout_window(self.scroll_vnode.clone().unwrap());
            }
            StandingsMessage::StandingsQueryReady(data) => {
                self.standings = data.unwrap_or_default();
            }
            StandingsMessage::SpawnPopoutStatic() => {
                let list = self
                .standings
                .standings
                .iter()
                .map(|(i, s, points, mwp, opp_mwp)| {
                        html! {
                            <tr>
                                <td>{ i }</td>
                                <td>{ s }</td>
                                <td>{ points }</td>
                                <td>{ format!( "{:.3}", rational_to_float(*mwp) ) }</td>
                                <td>{ format!( "{:.3}", rational_to_float(*opp_mwp) ) }</td>
                            </tr>
                        }
                })
                .collect::<Html>();
                let vnode = html!{
                    <table class="table">
                        <thead>
                            <tr>
                                <th>{ "Rank" }</th>
                                <th>{ "Player" }</th>
                                <th>{ "Points" }</th>
                                <th>{ "Match Win %" }</th>
                                <th>{ "Opponent Match Win %" }</th>
                            </tr>
                        </thead>
                        <tbody> { list } </tbody>
                    </table>
                };
                generic_popout_window(vnode.clone());
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = self.process.clone();
        let cb_static = ctx
            .link()
            .callback(move |_| StandingsMessage::SpawnPopoutStatic());
        html! {
            <div>
                <div class="overflow-auto py-3 pairings-scroll-box">
                    <ul class="force_left">{
                        self.standings.standings.iter().map(|(i, name, _points, _mwp, _opp_mwp)| {
                            html! {
                                <li>{ format!("{} : {}", i, name) }</li>
                            }
                        })
                        .collect::<Html>()
                    }</ul>
                </div>
                <button onclick={ move |_| cb.emit(0) }>{ "Standings Scroll" }</button>
                <button onclick={ cb_static}>{ "Standings Static" }</button>
            </div>
        }
    }
}

impl StandingsProfile {
    pub fn new(tourn: &Tournament) -> Self {
        let standings = tourn
            .get_standings()
            .scores
            .into_iter()
            .rev()
            .enumerate()
            .filter_map(|(i, (id, score))| {
                tourn
                    .player_reg
                    .get_player(&id)
                    .map(|p| (i+1, p.name.clone(), score.match_points, score.mwp, score.opp_mwp))
                    .ok()
            })
            .collect();
        Self { standings }
    }
}
