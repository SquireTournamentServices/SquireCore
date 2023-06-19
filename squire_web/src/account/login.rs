use yew::{functional::*, prelude::*};
use yew_router::prelude::*;

#[function_component(Login)]
pub fn login() -> Html {
    html! {
        <div>
            <p>{ "Log into an existing account" }</p>
        </div>
    }
}
