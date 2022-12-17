use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering}, thread, time::Duration,
};

use async_session::MemoryStore;
use axum::{Router, Server};
use once_cell::sync::OnceCell;
use tokio::io::{self, AsyncWriteExt};

use crate::{create_router, AppState, tournaments, accounts};

static SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static SERVER: OnceCell<()> = OnceCell::new();

async fn init() {
    accounts::init();
    tournaments::init();

    // `MemoryStore` is ephemeral and will not persist between test runs
    let app_state = AppState {
        store: MemoryStore::new(),
    };

    println!("Spawning server");

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    tokio::spawn(async move {
        println!("Creating server");
        axum::Server::bind(&addr)
            .serve(create_router(app_state).into_make_service())
            .await
            .unwrap();
    });
    
    tokio::time::sleep(Duration::from_millis(10)).await;

    println!("Setting server");
    SERVER.set(()).expect("Could not set initialized server")
}

pub(crate) async fn ensure_startup() -> () {
    println!("Ensuring server is starting!!");
    if !SERVER_STARTED
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
        .unwrap_or(true)
    {
        init().await;
    }
    println!("Waiting for server to start...");
    *SERVER.wait()
}
