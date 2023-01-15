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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use http::{header, request::Parts};

use squire_sdk::{
    accounts::{SquireAccount, SquireAccountId, VerificationData},
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
mod cards;

static VERSION_DATA: OnceCell<Version> = OnceCell::new();

pub async fn init() {
    VERSION_DATA.get_or_init(|| Version {
        version: "0.1.0-pre-alpha".to_string(),
        mode: ServerMode::Extended,
    });
    cards::init().await;
    accounts::init();
}

pub fn create_router(state: AppState) -> Router {
    server::create_router::<AppState>()
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
#[derive(Debug, Clone)]
pub struct AppState {
    store: MemoryStore,
}

#[async_trait]
impl ServerState for AppState {
    fn get_version(&self) -> Version {
        todo!()
    }

    fn get_verification_data(&self, user: &User) -> Option<VerificationData> {
        todo!()
    }

    async fn create_tournament(
        &self,
        user: User,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentManager {
        todo!()
    }

    async fn query_tournament<F, O>(&self, id: &TournamentId, f: F) -> Option<O>
    where
        F: Send + FnOnce(&TournamentManager) -> O,
    {
        todo!()
    }

    async fn create_verification_data(&self, user: &User) -> VerificationData {
        todo!()
    }

    async fn sync_tournament(
        &self,
        id: &TournamentId,
        user: &User,
        sync: OpSync,
    ) -> Option<SyncStatus> {
        todo!()
    }

    async fn rollback_tournament(
        &self,
        id: &TournamentId,
        user: &User,
        rollback: Rollback,
    ) -> Option<Result<(), RollbackError>> {
        todo!()
    }

    async fn load_user(&self, user: User) {
        todo!()
    }

    async fn get_user(&self, id: &SquireAccountId) -> Option<User> {
        todo!()
    }

    async fn get_cards_meta(&self) -> Meta {
        todo!()
    }

    async fn get_atomics(&self) -> Arc<Atomics> {
        todo!()
    }

    async fn update_cards(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

#[async_trait]
impl SessionStore for AppState {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> async_session::Result<Option<async_session::Session>> {
        self.store.load_session(cookie_value).await
    }

    async fn store_session(
        &self,
        session: async_session::Session,
    ) -> async_session::Result<Option<String>> {
        self.store.store_session(session).await
    }

    async fn destroy_session(&self, session: async_session::Session) -> async_session::Result {
        self.store.destroy_session(session).await
    }

    async fn clear_store(&self) -> async_session::Result {
        self.store.clear_store().await
    }
}
