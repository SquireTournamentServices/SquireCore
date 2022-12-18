#![feature(trivial_bounds)]
#![allow(unused)]

use std::{env, net::SocketAddr, time::Duration};

use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, RequestPartsExt, Router, TypedHeader,
};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use dashmap::DashMap;
use http::{header, request::Parts};
use tokio::sync::RwLock;

use squire_sdk::{
    accounts::SquireAccount,
    cards::atomics::Atomics,
    tournaments::{CreateTournamentRequest, CreateTournamentResponse},
};

static COOKIE_NAME: &str = "SESSION";

#[cfg(test)]
mod tests;

mod accounts;
mod cards;
mod tournaments;

use accounts::*;
use cards::MetaChecker;
use tournaments::*;

pub async fn init() {
    let atomics: Atomics = reqwest::get("https://mtgjson.com/api/v5/AtomicCards.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    cards::ATOMICS_MAP.set(RwLock::new(atomics)).unwrap();
    let meta: MetaChecker = reqwest::get("https://mtgjson.com/api/v5/Meta.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    cards::META_CACHE.set(RwLock::new(meta.meta)).unwrap();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1800));
        interval.tick().await;
        loop {
            interval.tick().await;
            cards::update_cards().await;
        }
    });

    accounts::init();

    tournaments::init();
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/tournaments", tournaments::get_routes())
        .nest("/api/v1", accounts::get_routes())
        .route("/api/v1/cards", get(cards::atomics))
        .route("/api/v1/meta", get(cards::meta))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_oauth=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    init().await;

    // `MemoryStore` is just used as an example. Don't use this in production.
    let app_state = AppState {
        store: MemoryStore::new(),
    };

    let app = create_router(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Starting server!!");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

#[derive(Clone)]
pub struct AppState {
    store: MemoryStore,
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    account: SquireAccount,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => StatusCode::FORBIDDEN,
                    _ => panic!("unexpected error getting Cookie header(s): {}", e),
                },
                _ => panic!("unexpected error getting cookies: {}", e),
            })?;

        let session_cookie = cookies.get(COOKIE_NAME).ok_or(StatusCode::FORBIDDEN)?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(StatusCode::FORBIDDEN)?;

        session.get("user").ok_or(StatusCode::FORBIDDEN)
    }
}
