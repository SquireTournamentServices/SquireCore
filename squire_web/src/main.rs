#![allow(non_camel_case_types)]
#![allow(dead_code, unused)]
#![feature(if_let_guard)]

use std::time::Duration;

use async_std::channel::{unbounded, Receiver};
use once_cell::sync::OnceCell;
use utils::console_log;
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
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Header />
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    web_sys::console::log_1(&format!("Starting everything up...").into());

    let (send, recv) = unbounded();
    let on_update = move || {
        //let _ = send.send(0);
        match (send.try_send(0)) {
            Ok(_) => console_log("Sent successfully"),
            Err(_) => console_log("Failed to send"),
        }
    };

    let client = SquireClient::new_unchecked(
        "/".to_string(),
        SquireAccount::new("Tester".into(), "Tester".into()),
        on_update,
    );
    CLIENT.set(client).unwrap();
    ON_UPDATE.set(recv).unwrap();
    web_sys::console::log_1(&format!("Client launched!! Starting yew app").into());
    yew::Renderer::<app>::new().render();
}
