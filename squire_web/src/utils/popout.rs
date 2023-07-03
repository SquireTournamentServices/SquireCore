use web_sys::window;
use yew::{prelude::*, virtual_dom::VNode};

#[function_component]
fn GeneralPopout(props: &GeneralPopoutProps) -> Html {
    props.display_vnode.clone()
}
#[derive(Properties, PartialEq)]
pub struct GeneralPopoutProps {
    pub display_vnode: VNode,
}

pub fn generic_popout_window(vnode: VNode) {
    window()
        .and_then(|w| w.open().ok().flatten())
        .and_then(|new_w_o| new_w_o.document())
        .and_then(|doc| doc.get_elements_by_tag_name("html").get_with_index(0))
        .map(|r| {
            yew::Renderer::<GeneralPopout>::with_root_and_props(
                r,
                GeneralPopoutProps {
                    display_vnode: vnode,
                },
            )
            .render()
        });
}

pub fn generic_scroll_vnode<I>(vert_scroll_time: u32, string_vec: I) -> Html
where
    I: Iterator<Item = String>,
{
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
                height: 2em;
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
                            string_vec
                            .map(|(s)|
                                html! {
                                    <div class="scroll_item">
                                        <p>{ format!("#{s}") }</p>
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
