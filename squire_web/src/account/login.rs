use std::borrow::Cow;

use derive_more::From;
use squire_sdk::{api::Credentials, client::network::LoginError, model::accounts::SquireAccount};
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDialogElement};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{utils::TextInput, Route, CLIENT};

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
    error_message: String,
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
            error_message: String::new(),
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
            LoginMessage::LoginResult(_) => {
                let element: HtmlDialogElement = window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("errormessage"))
                    .and_then(|e| e.dyn_into::<HtmlDialogElement>().ok())
                    .unwrap();
                self.error_message = "Login attempt failed!!".to_owned();
                let _ = element.show_modal();
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
        html! {
            <>
            <>
            <dialog id="errormessage">
                <p>{self.error_message.clone()}</p>
                <form method="dialog">
                <button>{"OK"}</button>
                </form>
            </dialog>
            </>
            <div class="m-lg-0 m-md-4 my-3">
                <div class="p-5 bg-light rounded-3">
                    <div class="container-fluid p-md-5">
                        <h1 class="display-5 fw-bold">{ "Login" }</h1>
                        <hr class="my-4"/>
                        <TextInput label = {Cow::from("Username ")} process = { name_callback } /><br class="my-2"/>
                        <TextInput label = {Cow::from("Password ")} process = { password_callback } /><br class="my-2"/>
                        <button onclick={submit_callback}>{ "Log in" }</button>
                    </div>
                </div>
            </div>
            </>
        }
    }
}
