#![allow(unused)]

use std::{net::SocketAddr, sync::Arc};

use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    http::StatusCode,
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use state::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use http::{header, request::Parts};

use squire_sdk::{
    accounts::{SquireAccount, SquireAccountId, VerificationData},
    api::{GET_ALL_PAST_TOURNAMENTS_ROUTE, TOURNAMENTS_ROUTE},
    cards::{atomics::Atomics, meta::Meta},
    response::SquireResponse,
    server::{self, state::ServerState, User},
    tournaments::{
        OpSync, Rollback, RollbackError, SyncStatus, TournamentId, TournamentManager,
        TournamentPreset,
    },
    version::{ServerMode, Version},
};

#[cfg(test)]
mod tests;

mod accounts;
mod tournaments;
//mod cards;
mod state;

pub async fn init() {
    //cards::init().await;
}

pub fn create_router(state: AppState) -> Router {
    server::create_router::<AppState>()
        .extend(TOURNAMENTS_ROUTE, tournaments::get_routes())
        //.route("/api/v1/cards", get(cards::atomics))
        //.route("/api/v1/meta", get(cards::meta))
        .into()
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

    let app_state = AppState::new().await;

    let app = create_router(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Starting server at: {addr:?}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
