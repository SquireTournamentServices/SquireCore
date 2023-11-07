use squire_sdk::{
    model::{identifiers::TournamentId, tournament::Tournament},
    sync::TournamentManager,
};
use yew::{prelude::*, virtual_dom::VNode};

use super::{
    InteractionResponse, TournQuery, TournViewerComponent, TournViewerComponentWrapper,
    WrapperMessage, WrapperState,
};
use crate::utils::{generic_popout_window, generic_scroll_vnode};

#[derive(Debug, PartialEq, Properties, Clone)]
struct StandingsPopoutProps {
    pub display_vnode: VNode,
}
#[function_component]
fn StandingsPopout(props: &StandingsPopoutProps) -> Html {
    props.display_vnode.clone()
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct StandingsProps {}
#[derive(Debug, PartialEq, Clone)]
pub enum StandingsMessage {
    SpawnPopout(i32),
}
pub enum StandingsQueryMessage {
    StandingsQueryReady(Option<StandingsProfile>),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StandingsProfile {
    standings: Vec<(usize, String)>,
}

pub struct StandingsView {
    pub id: TournamentId,
    pub scroll_vnode: Option<VNode>,
    standings: StandingsProfile,
}

impl TournViewerComponent for StandingsView {
    type InteractionMessage = StandingsMessage;
    type QueryMessage = StandingsQueryMessage;
    type Properties = StandingsProps;

    fn v_create(_ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self {
        StandingsView {
            id: state.t_id,
            scroll_vnode: None,
            standings: Default::default(),
        }
    }

    fn interaction(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _msg: Self::InteractionMessage,
        _state: &WrapperState,
    ) -> InteractionResponse<Self> {
        match _msg {
            StandingsMessage::SpawnPopout(_num) => {
                let scroll_strings = self
                    .standings
                    .standings
                    .iter()
                    .map(|(i, s)| format!("{i} : {s}"));
                self.scroll_vnode = Some(generic_scroll_vnode(120, scroll_strings));
                generic_popout_window(self.scroll_vnode.clone().unwrap());
            }
        }
        true.into()
    }

    fn load_queried_data(&mut self, _msg: Self::QueryMessage, _state: &WrapperState) -> bool {
        match _msg {
            StandingsQueryMessage::StandingsQueryReady(data) => {
                self.standings = data.unwrap_or_default();
                true
            }
        }
    }

    fn query(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> TournQuery<Self::QueryMessage> {
        let q_func = |tourn: &TournamentManager| {
            let data = StandingsProfile::new(tourn.tourn());
            Self::QueryMessage::StandingsQueryReady(Some(data))
        };
        Box::new(q_func)
    }

    fn v_view(
        &self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> yew::Html {
        let cb = _ctx
            .link()
            .callback(move |_| WrapperMessage::Interaction(StandingsMessage::SpawnPopout(20)));
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
                <button onclick={ cb }>{ "Standings Scroll" }</button>
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
