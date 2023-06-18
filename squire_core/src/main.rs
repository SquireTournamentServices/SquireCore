#![allow(unused)]

use std::{net::SocketAddr, sync::Arc};

use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use http::{header, request::Parts};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use state::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use squire_sdk::{
    api::TOURNAMENTS_ROUTE,
    model::accounts::SquireAccount,
    response::SquireResponse,
    server::{self, state::ServerState, User},
    tournaments::{
        OpSync, TournamentId, TournamentManager,
        TournamentPreset,
    },
    version::{ServerMode, Version},
};

#[cfg(test)]
mod tests;

//mod accounts;
mod assets;
mod state;
mod tournaments;

pub async fn init() {}

//#[axum::debug_handler]
pub fn create_router(state: AppState) -> Router {
    server::create_router::<AppState>(state.clone())
        //.extend(TOURNAMENTS_ROUTE, tournaments::get_routes())
        .into_router()
        .route("/", get(assets::landing))
        .route("/squire_web_bg.wasm", get(assets::get_wasm))
        .route("/squire_web.js", get(assets::get_js))
        .fallback(assets::landing)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    /*
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_oauth=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    */

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
