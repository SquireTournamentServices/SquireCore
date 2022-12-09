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

use squire_sdk::{accounts::SquireAccount, cards::atomics::Atomics, tournaments::{CreateTournamentResponse, CreateTournamentRequest}};

static COOKIE_NAME: &str = "SESSION";

#[cfg(test)]
mod tests;

//mod accounts;
//mod matches;
//mod players;
mod accounts;
mod cards;
mod tournaments;

//use players::*;
use accounts::*;
use cards::*;
use tournaments::*;

pub async fn init() -> Router {
    let atomics: Atomics = reqwest::get("https://mtgjson.com/api/v5/AtomicCards.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    ATOMICS_MAP.set(RwLock::new(atomics)).unwrap();
    let meta: MetaChecker = reqwest::get("https://mtgjson.com/api/v5/Meta.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    META_CACHE.set(RwLock::new(meta.meta)).unwrap();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1800));
        interval.tick().await;
        loop {
            interval.tick().await;
            update_cards().await;
        }
    });
    // `MemoryStore` is just used as an example. Don't use this in production.
    let app_state = AppState {
        store: MemoryStore::new(),
    };

    //let _ = USERS_MAP.set(DashMap::new());
    //let _ = ORGS_MAP.set(DashMap::new());
    let _ = TOURNS_MAP.set(DashMap::new());
    Router::new()
        .route("/api/v1/tournaments/create", post(create_tournament))
        .route("/api/v1/tournaments/:t_id", get(get_tournament))
        .route("/api/v1/tournaments/:t_id/sync", post(sync))
        .route("/api/v1/tournaments/:t_id/rollback", post(rollback))
        .route("/api/v1/cards", get(atomics))
        .route("/api/v1/meta", get(meta))
        .with_state(app_state)
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

    let _ = init().await;
    // `MemoryStore` is just used as an example. Don't use this in production.
    let app_state = AppState {
        store: MemoryStore::new(),
    };

    //let _ = USERS_MAP.set(DashMap::new());
    //let _ = ORGS_MAP.set(DashMap::new());
    let _ = TOURNS_MAP.set(DashMap::new());
    let app = Router::new()
        .route("/api/v1/tournaments/create", post(create_tournament))
        .route("/api/v1/tournaments/:t_id", get(get_tournament))
        .route("/api/v1/tournaments/:t_id/sync", post(sync))
        .route("/api/v1/tournaments/:t_id/rollback", post(rollback))
        .route("/api/v1/cards", get(atomics))
        .route("/api/v1/meta", get(meta))
        .with_state(app_state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Starting server!!");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

#[derive(Clone)]
struct AppState {
    store: MemoryStore,
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/auth/discord").into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct User {
    account: SquireAccount,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = AuthRedirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => AuthRedirect,
                    _ => panic!("unexpected error getting Cookie header(s): {}", e),
                },
                _ => panic!("unexpected error getting cookies: {}", e),
            })?;
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(AuthRedirect)?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(AuthRedirect)?;

        let user = session.get::<User>("user").ok_or(AuthRedirect)?;

        Ok(user)
    }
}

#[axum::debug_handler]
pub async fn create_tournament(user: User, Json(data): Json<CreateTournamentRequest>) -> CreateTournamentResponse {
    let tourn = user.account.create_tournament(data.name, data.preset, data.format);
    let id = tourn.id;
    TOURNS_MAP.get().unwrap().insert(id, tourn.clone());
    CreateTournamentResponse::new(tourn)
}

