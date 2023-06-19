use squire_sdk::tournaments::{StandardScore, Standings, Tournament, TournamentId};
use web_sys::window;
use yew::{prelude::*, virtual_dom::VNode};

use self::_StandingsPopoutProps::display_vnode;
use crate::{tournament::standings, CLIENT};

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
        StandingsView {
            id: ctx.props().id,
            scroll_vnode: None,
            process: ctx.link().callback(StandingsMessage::SpawnPopout),
            standings: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            StandingsMessage::SpawnPopout(num) => {
                self.scroll_vnode = Some(self.standings.gen_popout_page(120));
                window()
                    .and_then(|w| w.open().ok().flatten())
                    .and_then(|new_w_o| new_w_o.document())
                    .and_then(|doc| doc.get_elements_by_tag_name("html").get_with_index(0))
                    .map(|r| {
                        yew::Renderer::<StandingsPopout>::with_root_and_props(
                            r,
                            StandingsPopoutProps {
                                display_vnode: self.scroll_vnode.clone().unwrap(),
                            },
                        )
                        .render()
                    });
            }
            StandingsMessage::StandingsQueryReady(data) => {
                self.standings = data.unwrap_or_default();
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = self.process.clone();
        html! {
            <div>
                <p>{ self.standings.standings.len() }</p>
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
            .filter_map(|(i, (id, score))| {
                tourn
                    .player_reg
                    .get_player(&id)
                    .map(|p| (i, p.name.clone()))
                    .ok()
            })
            .collect();
        Self { standings }
    }

    pub fn view(&self) -> Html {
        todo!()
    }

    fn gen_popout_page(&self, vert_scroll_time: u32) -> Html {
        html! {
            <html>
                <head>
                    <title>{ "Standings!" }</title>
                    <style>{format!("
                html, body {{
                    overflow: hidden;
                }}
                .scroll_holder {{
                    overflow: hidden;
                    box-sizing: border-box;
                    position: relative;
                    box-sizing: border-box;
                }}
                .vert_scroller {{
                    position: relative;
                    box-sizing: border-box;
                    animation: vert_scroller {}s linear infinite;
                }}
                .scroll_item {{
                    display: block;
                    font-size: 3em;
                    font-family: Arial, Helvetica, sans-serif;
                    margin: auto;
                    height: 4em;
                    text-align: center;
                }}
                @keyframes vert_scroller {{
                    0%   {{ top:  120vh }}
                    100% {{ top: -100% }}
                }}
                ", vert_scroll_time)}</style>
                </head>
                <body>
                    <div class="scroll_holder">
                        <div class="vert_scroller">
                        {
                                self.standings
                                .iter()
                                .map(|(i, name)|
                                    html! {
                                        <div class="scroll_item">
                                            <p>{ format!("#{i} : {name}") }</p>
                                        </div>
                                    })
                                .collect::<Html>()
                        }
                        </div>
                    </div>
                </body>
            </html>
        }
    }
}
