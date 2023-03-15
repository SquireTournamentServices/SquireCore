use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::utils::TextInput;

pub enum RegisterMessage {
    NameInput(String),
    DisplayInput(String),
}

pub struct Register {
    input: (Option<String>, Option<String>),
}

impl Component for Register {
    type Message = RegisterMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: (None, None)
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RegisterMessage::NameInput(s) => {
                self.input.0 = Some(s);
            },
            RegisterMessage::DisplayInput(s) => {
                self.input.1 = Some(s);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name_callback = ctx.link().callback(RegisterMessage::NameInput);
        let display_callback = ctx.link().callback(RegisterMessage::DisplayInput);
        let form = html! {
            <div>
                <TextInput label = {"Data"} process = { name_callback }/>
                <TextInput label = {"Data"} process = { display_callback }/>
            </div>
        };
        let data = match &self.input {
            (Some(name), Some(display)) => {
                html! {
                    <div>
                        <p>{ format!("Hello {name}! '{display}' is a great user name!") }</p>
                    </div>
                }
            },
            _ => Default::default()
        };
        html! {
            <div>
                { form }
                { data }
            </div>
        }
    }
}
