use axum::Router;
use once_cell::sync::OnceCell;
use tokio::sync::{Mutex, MutexGuard};

use crate::{create_router, AppState};

static SERVER: OnceCell<Mutex<Router>> = OnceCell::new();

fn init(app_state: AppState) -> Mutex<Router> {
    Mutex::new(create_router(app_state))
}

pub(crate) async fn get_app() -> MutexGuard<'static, Router> {
    let app = match SERVER.get() {
        Some(app) => app,
        None => {
            let app_state = AppState::new().await;
            SERVER.get_or_init(|| init(app_state))
        }
    };
    app.lock().await
}
