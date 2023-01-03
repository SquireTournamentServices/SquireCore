#![allow(unused)]

use std::{net::SocketAddr, sync::Arc};

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

use self::state::ServerState;

static COOKIE_NAME: &str = "SESSION";

#[cfg(test)]
mod tests;

pub mod accounts;
//mod cards;
pub mod state;
pub mod tournaments;

pub fn create_router<S>() -> Router<S>
where
    S: ServerState,
{
    Router::new()
        .nest(
            "/api/v1/tournaments",
            tournaments::get_routes::<S>(),
        )
        .nest("/api/v1", accounts::get_routes::<S>())
        .route("/api/v1/version", get(|| async { "0.1.0-pre-alpha" }))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    account: SquireAccount,
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
