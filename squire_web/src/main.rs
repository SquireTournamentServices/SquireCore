#![allow(non_camel_case_types)]
#![allow(dead_code, unused)]
#![feature(if_let_guard)]

use std::time::Duration;

use once_cell::sync::OnceCell;
use yew::prelude::*;
use yew_router::prelude::*;

use squire_sdk::{accounts::SquireAccount, client::SquireClient, tournaments::TournamentId};

mod account;
mod header;
mod index;
mod tournament;
mod utils;

use account::{Login, Register};
use header::Header;
use index::Index;
use tournament::{creator::TournamentCreator, viewer::TournamentViewer};

static CLIENT: OnceCell<SquireClient> = OnceCell::new();

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
            <Header />
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    let client = SquireClient::new_unchecked(
        "/".to_string(),
        SquireAccount::new("Tester".into(), "Tester".into()),
    );
    CLIENT.set(client).unwrap();
    yew::Renderer::<app>::new().render();
}
