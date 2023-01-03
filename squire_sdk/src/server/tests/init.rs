use std::{sync::Arc, collections::{HashMap, HashSet}};

use async_session::MemoryStore;
use axum::Router;
use dashmap::DashMap;
use once_cell::sync::OnceCell;

use tokio::sync::{Mutex, MutexGuard};

use crate::server::{accounts, create_router, tournaments};

use super::AppState;

static SERVER: OnceCell<Mutex<Router>> = OnceCell::new();

fn init() -> Mutex<Router> {
    // `MemoryStore` is ephemeral and will not persist between test runs
    let app_state = AppState {
        store: MemoryStore::new(),
        users: DashMap::new(),
        verified: DashMap::new(),
        tourns: DashMap::new(),
    };

    Mutex::new(create_router::<AppState>().with_state(app_state))
}

pub(crate) async fn get_app() -> MutexGuard<'static, Router> {
    SERVER.get_or_init(init).lock().await
}
