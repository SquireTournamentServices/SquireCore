use async_session::MemoryStore;
use axum::Router;
use once_cell::sync::OnceCell;

use tokio::sync::{Mutex, MutexGuard};

use crate::{accounts, create_router, tournaments, AppState, MainAppState};

static SERVER: OnceCell<Mutex<Router>> = OnceCell::new();

fn init() -> Mutex<Router> {
    accounts::init();
    tournaments::init();

    // `MemoryStore` is ephemeral and will not persist between test runs
    let app_state = MainAppState {
        store: MemoryStore::new(),
    };

    Mutex::new(create_router(AppState::Main(app_state)))
}

pub(crate) async fn get_app() -> MutexGuard<'static, Router> {
    SERVER.get_or_init(init).lock().await
}
