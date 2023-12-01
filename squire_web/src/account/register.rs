use std::borrow::Cow;

use squire_sdk::api::RegForm;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDialogElement, window};
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;

// use yew_router::prelude::*;
use crate::{utils::TextInput, CLIENT, Route};

pub enum RegisterMessage {
    NameInput(String),
    DisplayInput(String),
    PasswordInput(String),
    RePasswordInput(String),
    SubmitRegister,
    RegisterResult(Result<bool, reqwest::Error>),
}

pub struct Register {
    input: (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    error_message : String
}

impl Register {
    fn display_error(&mut self, err: String) {
        let element: HtmlDialogElement = window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("errormessage"))
        .and_then(|e| e.dyn_into::<HtmlDialogElement>().ok())
        .unwrap();
        self.error_message = err;
        let _ = element.show_modal();
    }
    fn get_regform(&self) -> Result<RegForm, String> {
        let name = self.input.0.clone().unwrap_or_else(|| "".to_owned());
        if name.is_empty() {
            return Err("You need to enter a username".to_owned());
        }
        let display = self.input.1.clone().unwrap_or_else(|| "".to_owned());
        if display.is_empty() {
            return Err("You need to enter a display name.".to_owned());
        }
        let password = self.input.2.clone().unwrap_or_else(|| "".to_owned());
        if password.is_empty() {
            return Err("You need to enter a password".to_owned());
        }
        let repassword = self.input.3.clone().unwrap_or_else(|| "".to_owned());
        if repassword.is_empty() {
            return Err("You need to re-enter your password".to_owned());
        }
        if password != repassword {
            return Err("Your entered passwords need to match.".to_owned());
        }
        Ok(RegForm {
            username: name,
            display_name: display,
            password: password,
        })
    }
}

impl Component for Register {
    type Message = RegisterMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: (None, None, None, None),
            error_message : String::new()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RegisterMessage::NameInput(s) => self.input.0 = Some(s),
            RegisterMessage::DisplayInput(s) => self.input.1 = Some(s),
            RegisterMessage::PasswordInput(s) => self.input.2 = Some(s),
            RegisterMessage::RePasswordInput(s) => self.input.3 = Some(s),
            RegisterMessage::SubmitRegister => {
                let client = CLIENT.get().unwrap();
                let reg_result = self.get_regform();
                match reg_result {
                    Ok(regform) => {
                        ctx.link().send_future(client.register(regform).output());
                    }
                    Err(err) => {
                        self.display_error(err);
                    }
                }
                return false;
            }
            RegisterMessage::RegisterResult(Ok(_)) => {
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::Create);
            }
            RegisterMessage::RegisterResult(_) => {
                self.display_error("Failed to regeister".to_string());
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // let navigator = ctx.link().navigator().unwrap();
        /*
        let submit_callback = move |_| {
            RegisterMessage::SubmitRegister);
        */
        let submit_callback = ctx.link().callback(|_| RegisterMessage::SubmitRegister);
        let name_callback = ctx.link().callback(RegisterMessage::NameInput);
        let display_callback = ctx.link().callback(RegisterMessage::DisplayInput);
        let password_callback = ctx.link().callback(RegisterMessage::PasswordInput);
        let repassword_callback = ctx.link().callback(RegisterMessage::RePasswordInput);
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
                        <h1 class="display-5 fw-bold">{ "Register" }</h1>
                        <hr class="my-4"/>
                        <TextInput label = {Cow::from("Username ")} process = { name_callback } /><br class="my-2"/>
                        <TextInput label = {Cow::from("Display name ")} process = { display_callback } /><br class="my-2"/>
                        <TextInput label = {Cow::from("Password ")} process = { password_callback } /><br class="my-2"/>
                        <TextInput label = {Cow::from("Re-Type Password ")} process = { repassword_callback } /><br class="my-2"/>
                        <button onclick={submit_callback}>{ "Register" }</button>
                    </div>
                </div>
            </div>
            </>
        }
    }
}

impl From<Result<bool, reqwest::Error>> for RegisterMessage {
    fn from(value: Result<bool, reqwest::Error>) -> Self {
        Self::RegisterResult(value)
    }
}
