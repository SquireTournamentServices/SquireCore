use std::borrow::Cow;

use yew::{functional::*, prelude::*};
use yew_router::prelude::*;

use crate::{utils::TextInput, Route};

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
            input: (None, None),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RegisterMessage::NameInput(s) => {
                self.input.0 = Some(s);
            }
            RegisterMessage::DisplayInput(s) => {
                self.input.1 = Some(s);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let do_submit = self.input.0.is_some() && self.input.1.is_some();
        let navigator = ctx.link().navigator().unwrap();
        let submit_callback = Callback::from(move |_| {
            if do_submit {
                navigator.push(&Route::Create);
            }
        });
        let name_callback = ctx.link().callback(RegisterMessage::NameInput);
        let display_callback = ctx.link().callback(RegisterMessage::DisplayInput);
        let form = html! {
            <div>
                <TextInput label = {Cow::from("Your name")} process = { name_callback }/>
                <TextInput label = {Cow::from("Display name")} process = { display_callback }/>
            </div>
        };
        html! {
            <div>
                { form }
                <button onclick={submit_callback}>{ "Register" }</button>
            </div>
        }
    }
}
