use axum::Router;
use once_cell::sync::OnceCell;

use tokio::sync::{Mutex, MutexGuard};

use squire_sdk::server::create_router;

/*
use super::AppState;

static SERVER: OnceCell<Mutex<Router>> = OnceCell::new();

fn init() -> Mutex<Router> {
    let app_state = AppState::new();
    Mutex::new(create_router(app_state.clone()).into_router())
}

pub(crate) async fn get_app() -> MutexGuard<'static, Router> {
    SERVER.get_or_init(init).lock().await
}
*/
