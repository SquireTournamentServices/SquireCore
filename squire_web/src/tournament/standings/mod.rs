use web_sys::window;
use yew::{prelude::*, virtual_dom::VNode};

use crate::{tournament::standings, CLIENT};
use squire_sdk::tournaments::{StandardScore, Standings, Tournament, TournamentId};

use self::_StandingsPopoutProps::display_vnode;

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
    SpawnPopout(i32),
}

pub struct StandingsView {
    pub id: TournamentId,
    pub scroll_vnode: Option<VNode>,
    pub process: Callback<i32>,
}

fn gen_popout_page(standings: Vec<(usize, String)>, vert_scroll_time: u32) -> Html {
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
                            standings
                            .into_iter()
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

impl Component for StandingsView {
    type Message = StandingsMessage;
    type Properties = StandingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        StandingsView {
            id: ctx.props().id,
            scroll_vnode: None,
            process: ctx.link().callback(StandingsMessage::SpawnPopout),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            StandingsMessage::SpawnPopout(num) => {
                let standings = CLIENT
                    .get()
                    .unwrap()
                    .query_tourn(self.id, |t| {
                        t.get_standings()
                            .scores
                            .into_iter()
                            .map(|(p, _)| {
                                t.get_player(&p.into())
                                    .map(|p| p.name)
                                    .unwrap_or_else(|_| "Not Found".to_owned())
                            })
                            .enumerate()
                            .collect::<Vec<_>>()
                    })
                    .process();
                self.scroll_vnode = standings.map(|s| gen_popout_page(s, 120));
                window().map(|w| {
                    w.open().map(|new_w_o| {
                        new_w_o.map(|new_w| {
                            new_w.document().map(|doc| {
                                doc.get_elements_by_tag_name("html")
                                    .get_with_index(0)
                                    .map(|r| {
                                        yew::Renderer::<StandingsPopout>::with_root_and_props(
                                            r.into(),
                                            StandingsPopoutProps {
                                                display_vnode: self.scroll_vnode.clone().unwrap(),
                                            },
                                        )
                                        .render()
                                    })
                            })
                        })
                    })
                });
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let len = CLIENT
            .get()
            .unwrap()
            .query_tourn(self.id, |t| t.get_standings().scores.len())
            .process()
            .unwrap_or_default();
        let cb = self.process.clone();
        html! {
            <div>
                <p>{ len }</p>
                <button onclick={ move |_| cb.emit(0) }>{ "Standings Scroll" }</button>
            </div>
        }
    }
}
