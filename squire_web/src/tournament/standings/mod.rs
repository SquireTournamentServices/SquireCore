use squire_sdk::model::{identifiers::TournamentId, tournament::Tournament};
use yew::{prelude::*, virtual_dom::VNode};

use crate::{
    utils::{console_log, generic_popout_window, generic_scroll_vnode},
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
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StandingsProfile {
    standings: Vec<(usize, String)>,
}

pub struct StandingsView {
    pub id: TournamentId,
    pub scroll_vnode: Option<VNode>,
    pub process: Callback<i32>,
    standings: StandingsProfile,
}

pub fn fetch_standings_profile(ctx: &Context<StandingsView>, id: TournamentId) {
    console_log("Standings are being fetched...");
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
                    .map(|(i, s)| format!("{i} : {s}"));
                self.scroll_vnode = Some(generic_scroll_vnode(120, scroll_strings));
                generic_popout_window(self.scroll_vnode.clone().unwrap());
            }
            StandingsMessage::StandingsQueryReady(data) => {
                self.standings = data.unwrap_or_default();
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let cb = self.process.clone();
        html! {
            <div>
                <div class="overflow-auto py-3 pairings-scroll-box">
                    <ul class="force_left">{
                        self.standings.standings.iter().map(|(i, name)| {
                            html! {
                                <li>{ format!("{} : {}", i, name) }</li>
                            }
                        })
                        .collect::<Html>()
                    }</ul>
                </div>
                <button onclick={ move |_| cb.emit(0) }>{ "Standings Scroll" }</button>
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
            .enumerate()
            .filter_map(|(i, (id, _score))| {
                tourn
                    .player_reg
                    .get_player(&id)
                    .map(|p| (i, p.name.clone()))
                    .ok()
            })
            .collect();
        Self { standings }
    }
}
