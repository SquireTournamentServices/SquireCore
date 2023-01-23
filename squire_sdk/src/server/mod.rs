#![allow(unused)]

use std::{net::SocketAddr, sync::Arc};

use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts, State},
    http::StatusCode,
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use serde::{Deserialize, Serialize};
use squire_lib::accounts::SquireAccount;

use http::{header, request::Parts};

use crate::{
    api::{ACCOUNTS_ROUTE, TOURNAMENTS_ROUTE},
    version::ServerVersionResponse,
    COOKIE_NAME,
};

use self::state::ServerState;

pub mod accounts;
//mod cards;
pub mod state;
pub mod tournaments;

pub fn create_router<S>() -> Router<S>
where
    S: ServerState,
{
    Router::new()
        .nest(TOURNAMENTS_ROUTE, tournaments::get_routes::<S>())
        .nest(ACCOUNTS_ROUTE, accounts::get_routes::<S>())
        .route("/api/v1/version", get(get_version::<S>))
}

pub async fn get_version<S>(State(state): State<S>) -> ServerVersionResponse
where
    S: ServerState,
{
    ServerVersionResponse::new(state.get_version())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub account: SquireAccount,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: ServerState,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
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

        println!("Looking for correct cookie:\n{cookies:?}");
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(StatusCode::FORBIDDEN)?;

        println!("Loading Session...");
        let session = state
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(StatusCode::FORBIDDEN)?;
        session.expire_in(std::time::Duration::from_secs(600));
        println!("Session loaded successfully!");

        session.get("user").ok_or(StatusCode::FORBIDDEN);
        if session.is_expired() {
            session.destroy();
            Ok(())
        }
    }
}
