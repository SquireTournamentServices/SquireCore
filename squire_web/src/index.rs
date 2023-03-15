use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

#[function_component(Index)]
pub fn index() -> Html {
    let navigator = use_navigator().unwrap();
    let register_callback = Callback::from(move |_| {
        navigator.push(&Route::Register)
    });

    let navigator = use_navigator().unwrap();
    let login_callback = Callback::from(move |_| {
        navigator.push(&Route::Login)
    });

    html! {
        <div>
            <p>{ "What would you like to do?" }</p>
            <button onclick={register_callback}>{ "Register" }</button>
            <button onclick={login_callback}>{ "Login" }</button>
        </div>
    }
}
