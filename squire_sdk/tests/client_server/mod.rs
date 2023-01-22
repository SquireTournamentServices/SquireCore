use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use once_cell::sync::OnceCell;
use squire_sdk::{
    accounts::SquireAccount,
    client::{simple_state::SimpleState, SquireClient},
    server::User,
};
use tokio::time::interval;

use crate::server::AppState;

pub mod startup;
pub mod tournaments;

static STARTING_UP: AtomicBool = AtomicBool::new(false);
static SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static CLIENT: OnceCell<SquireClient<SimpleState>> = OnceCell::new();

pub async fn init() {
    let account = SquireAccount::new("Test User".to_owned(), "Test User".to_owned());
    let user = User {
        account: account.clone(),
    };
    if let Ok(false) =
        SERVER_STARTED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
    {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
        tokio::spawn(async move {
            let state = AppState::new();
            state.users.insert(user.account.id, user);
            let app = squire_sdk::server::create_router().into().with_state(state);
            if let Err(_) = axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
            {
                SERVER_STARTED.store(false, Ordering::Relaxed);
                println!("Could not start server");
            }
        });
    }
    tokio::time::sleep(Duration::from_millis(1)).await;
    match SquireClient::new(
        "http://localhost:8000".to_owned(),
        account,
        SimpleState::new(),
    )
    .await
    {
        Ok(client) => {
            CLIENT.set(client).unwrap();
        }
        Err(err) => {
            println!("Could not connect client: {err:?}");
        }
    }
    STARTING_UP.store(false, Ordering::Relaxed);
}

pub async fn get_client() -> SquireClient<SimpleState> {
    let mut counter = 0;
    let mut timer = interval(Duration::from_millis(10));
    loop {
        if let Some(false) = STARTING_UP
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .ok()
            .and_then(|b| Some(b || CLIENT.get().is_some()))
        {
            init().await;
        }
        match CLIENT.get() {
            Some(c) => return c.clone(),
            None => {
                counter += 1;
            }
        }
        timer.tick().await;
        if counter == 20 {
            panic!("Unable to start server *and* establish client");
        }
    }
}
