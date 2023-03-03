#![allow(non_camel_case_types)]
#![allow(dead_code, unused)]

use once_cell::sync::OnceCell;
use tournament::{creator::TournamentCreator, viewer::TournamentViewer};
use yew::prelude::*;
use yew_router::prelude::*;

use squire_sdk::{accounts::SquireAccount, client::SquireClient, tournaments::TournamentId};

mod client;
mod tournament;

use client::WebState;

static CLIENT: OnceCell<SquireClient<WebState>> = OnceCell::new();

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Create,
    #[at("/:id")]
    Tourn { id: TournamentId },
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Create => html! { <TournamentCreator /> },
        Route::Tourn { id } => html! { <TournamentViewer id = { id } /> },
    }
}

#[function_component]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    let state = WebState::new();
    let client = SquireClient::new_unchecked(
        "localhost:8888".to_string(),
        SquireAccount::new("Tester".into(), "Tester".into()),
        state,
    );
    CLIENT.set(client).unwrap();
    yew::Renderer::<app>::new().render();
}
