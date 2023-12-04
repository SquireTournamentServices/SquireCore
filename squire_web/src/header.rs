use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::CLIENT;

#[function_component(Header)]
pub fn header() -> Html {
    let nav = use_navigator().unwrap();
    let make_route = move |route| {
        let nav = nav.clone();
        Callback::from(move |_| nav.push(&route))
    };
    let account_option = CLIENT.get().unwrap().get_user();
    let account_nav = match account_option {
        Some(_) => {
            html! {
                <>
                <li class="nav-item">
                <a class="nav-link" onclick = { make_route(Route::Login) }>{ "Login" }</a>
                </li>
                <li class="nav-item">
                    <a class="nav-link" onclick = { make_route(Route::Register) }>{ "Register" }</a>
                </li>
                </>
            }
        }
        None => {
            html! {
                <>
                <li class="nav-item">
                <a class="nav-link" onclick = { make_route(Route::Account) }>{ "Profile" }</a>
                </li>
                </>
            }
        }
    };
    html! {
        <header>
            <nav class="navbar navbar-expand-md navbar-dark bg-dark">
                <div class="container-fluid">
                    <a class="navbar-brand" href="#">{ "Squire Web" }</a>
                    <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarCollapse" aria-controls="navbarCollapse" aria-expanded="false" aria-label="Toggle navigation">
                        <span class="navbar-toggler-icon"></span>
                    </button>
                    <div class="collapse navbar-collapse" id="navbarCollapse">
                        <ul class="navbar-nav ms-auto">
                            <>{ account_nav }</>
                            <li class="nav-item">
                                <a class="nav-link" onclick = { make_route(Route::Create) }>{ "Create Tournament" }</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link">{ "View Tournaments" }</a>
                            </li>
                        </ul>
                    </div>
                </div>
            </nav>
        </header>
    }
}
