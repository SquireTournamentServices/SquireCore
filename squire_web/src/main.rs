#![allow(non_camel_case_types)]
#![allow(dead_code, unused)]
#![feature(if_let_guard)]

use once_cell::sync::OnceCell;
use yew::prelude::*;
use yew_router::prelude::*;

use squire_sdk::{accounts::SquireAccount, client::SquireClient, tournaments::TournamentId};

mod account;
mod client;
mod index;
mod header;
mod tournament;
mod utils;

use account::{Login, Register};
use client::WebState;
use index::Index;
use header::Header;
use tournament::{creator::TournamentCreator, viewer::TournamentViewer};

static CLIENT: OnceCell<SquireClient<WebState>> = OnceCell::new();

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Index,
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/create")]
    Create,
    #[at("/:id")]
    Tourn { id: TournamentId },
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Index => html! { <Index /> },
        Route::Login => html! { <Login /> },
        Route::Register => html! { <Register /> },
        Route::Create => html! { <TournamentCreator /> },
        Route::Tourn { id } => html! { <TournamentViewer id = { id } /> },
    }
}

#[function_component]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <header>
                <nav class="navbar navbar-expand-md navbar-dark bg-dark">
                    <div class="container-fluid">
                        <a class="navbar-brand" href="#">{ "Squire Web" }</a>
                        <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarCollapse" aria-controls="navbarCollapse" aria-expanded="false" aria-label="Toggle navigation">
                            <span class="navbar-toggler-icon"></span>
                        </button>
                        <div class="collapse navbar-collapse" id="navbarCollapse">
                            <ul class="navbar-nav ms-auto">
                                <li class="nav-item">
                                    <a class="nav-link">{ "Login" }</a>
                                </li>
                                <li class="nav-item">
                                    <a class="nav-link">{ "Register" }</a>
                                </li>
                                <li class="nav-item">
                                    <a class="nav-link">{ "Create Tournament" }</a>
                                </li>
                                <li class="nav-item">
                                    <a class="nav-link">{ "Veiw Tournaments" }</a>
                                </li>
                            </ul>
                        </div>
                    </div>
                </nav>
            </header>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    let state = WebState::new();
    let client = SquireClient::new_unchecked(
        "/".to_string(),
        SquireAccount::new("Tester".into(), "Tester".into()),
        state,
    );
    CLIENT.set(client).unwrap();
    yew::Renderer::<app>::new().render();
}
