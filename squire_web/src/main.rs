#![deny(non_camel_case_types)]
#![deny(dead_code, unused)]
#![feature(if_let_guard)]

use async_std::channel::{unbounded, Receiver};
use once_cell::sync::OnceCell;
use squire_sdk::{
    client::SquireClient,
    model::{accounts::SquireAccount, identifiers::TournamentId},
};
use yew::prelude::*;
use yew_router::prelude::*;

mod account;
mod header;
mod index;
mod tournament;
mod utils;

use account::{Login, Register};
use header::Header;
use index::Index;
use tournament::{creator::TournamentCreator, viewer::TournamentViewer};

/// The SquireClient used to manage tournaments and communicate with the backend
static CLIENT: OnceCell<SquireClient> = OnceCell::new();
/// The Receiver half of the channel used to communicate that the client has updated a tournament.
pub static ON_UPDATE: OnceCell<Receiver<usize>> = OnceCell::new();

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
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Header />
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    let (send, recv) = unbounded();
    let on_update = move |_| {
        let _ = send.try_send(0);
    };

    let client = SquireClient::builder()
        .url("/".to_string())
        .account(SquireAccount::new("Tester".into(), "Tester".into()))
        .on_update(on_update)
        .build_unchecked();
    CLIENT.set(client).unwrap();
    ON_UPDATE.set(recv).unwrap();
    yew::Renderer::<App>::new().render();
}
