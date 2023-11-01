use std::borrow::Cow;

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

pub enum LoginMessage {
    NameInput(String),
    PasswordInput(String),
    SubmitLogin,
}

pub struct Login {
    input: (Option<String>, Option<String>),
}

impl Login {
    fn _get_logform(&self) -> Result<String, String> {
        todo!()
    }
}

impl Component for Login {
    type Message = LoginMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: (None, None),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LoginMessage::NameInput(s) => self.input.0 = Some(s),
            LoginMessage::PasswordInput(s) => self.input.1 = Some(s),
            LoginMessage::SubmitLogin => {
                let _client = CLIENT.get().unwrap();
                todo!();
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit_callback = ctx.link().callback(|_| LoginMessage::SubmitLogin);
        let name_callback = ctx.link().callback(LoginMessage::NameInput);
        let password_callback = ctx.link().callback(LoginMessage::PasswordInput);
        let form = html! {
            <div>
                <TextInput label = {Cow::from("Username")} process = { name_callback } />
                <TextInput label = {Cow::from("Password")} process = { password_callback } />
            </div>
        };
        html! {
            <div>
                { form }
                <button onclick={submit_callback}>{ "Log in" }</button>
            </div>
        }
    }
}
