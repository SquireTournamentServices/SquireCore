#![allow(unused)]

use std::net::SocketAddr;

use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    http::StatusCode,
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use serde::{Deserialize, Serialize};
use squire_lib::accounts::SquireAccount;

use http::{header, request::Parts};

static COOKIE_NAME: &str = "SESSION";

#[cfg(test)]
mod tests;

pub mod accounts;
//mod cards;
pub mod state;
pub mod tournaments;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/tournaments", tournaments::get_routes())
        .nest("/api/v1", accounts::get_routes())
        .route("/api/v1/version", get(|| async { "0.1.0-pre-alpha" }))
        .with_state(state)
}

#[derive(Debug, Clone)]
pub enum AppState {
    Main(MainAppState),
    //Other(Box<dyn ServerState>),
}

#[derive(Debug, Clone)]
pub struct MainAppState {
    store: MemoryStore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    account: SquireAccount,
}

#[async_trait]
impl FromRequestParts<AppState> for User {
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        println!("Loading Cookies from parts...");
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

        println!("Looking for correct cookie...");
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(StatusCode::FORBIDDEN)?;

        println!("Loading Session...");
        let session = state
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(StatusCode::FORBIDDEN)?;
        println!("Session loaded successfully!");

        session.get("user").ok_or(StatusCode::FORBIDDEN)
    }
}

#[async_trait]
impl SessionStore for AppState {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> async_session::Result<Option<async_session::Session>> {
        match self {
            AppState::Main(state) => state.store.load_session(cookie_value).await,
            //AppState::Other(state) => state.load_session(cookie_value).await,
        }
    }

    async fn store_session(
        &self,
        session: async_session::Session,
    ) -> async_session::Result<Option<String>> {
        match self {
            AppState::Main(state) => state.store.store_session(session).await,
            //AppState::Other(state) => state.store_session(session).await,
        }
    }

    async fn destroy_session(&self, session: async_session::Session) -> async_session::Result {
        match self {
            AppState::Main(state) => state.store.destroy_session(session).await,
            //AppState::Other(state) => state.destroy_session(session).await,
        }
    }

    async fn clear_store(&self) -> async_session::Result {
        match self {
            AppState::Main(state) => state.store.clear_store().await,
            //AppState::Other(state) => state.clear_store().await,
        }
    }
}
