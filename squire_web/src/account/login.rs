use std::borrow::Cow;

use derive_more::From;
use squire_sdk::{api::Credentials, client::network::LoginError, model::accounts::SquireAccount};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{utils::TextInput, CLIENT, Route};

#[derive(Debug, From)]
pub enum LoginMessage {
    #[from(ignore)]
    NameInput(String),
    #[from(ignore)]
    PasswordInput(String),
    SubmitLogin,
    LoginResult(Result<SquireAccount, LoginError>),
}

pub struct Login {
    username: String,
    password: String,
}

impl Login {
    fn get_cred(&self) -> Credentials {
        Credentials::Basic {
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

impl Component for Login {
    type Message = LoginMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            username: String::new(),
            password: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LoginMessage::NameInput(s) => self.username = s,
            LoginMessage::PasswordInput(s) => self.password = s,
            LoginMessage::SubmitLogin => {
                let tracker = CLIENT.get().unwrap().login(self.get_cred());
                ctx.link().send_future(tracker);
            }
            LoginMessage::LoginResult(Ok(_)) => {
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::Create);
            }
            LoginMessage::LoginResult(_) => panic!("Login attempt failed!!"),
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
