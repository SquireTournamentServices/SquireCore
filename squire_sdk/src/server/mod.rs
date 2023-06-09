#![allow(unused)]

use std::{
    collections::{hash_map::Entry, HashMap},
    net::SocketAddr,
    sync::Arc,
};

use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts, State},
    http::StatusCode,
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use http::{header, request::Parts};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use squire_lib::{
    accounts::{SharingPermissions, SquireAccount},
    identifiers::SquireAccountId,
};

use crate::{
    api::{ACCOUNTS_ROUTE, API_BASE, TOURNAMENTS_ROUTE, VERSION_ENDPOINT, VERSION_ROUTE},
    utils::Url,
    version::ServerVersionResponse,
    COOKIE_NAME,
};

use self::{gathering::init_gathering_hall, state::ServerState};

//pub mod accounts;
//mod cards;
pub mod gathering;
pub mod state;
pub mod tournaments;

fn get_routes<S>() -> Router<S>
where
    S: ServerState,
{
    Router::new().route(VERSION_ENDPOINT.as_str(), get(get_version::<S>))
}

pub fn create_router<S>(state: S) -> SquireRouter<S>
where
    S: ServerState,
{
    init_gathering_hall(state);
    SquireRouter::new()
        .extend(API_BASE, get_routes::<S>())
        .extend(TOURNAMENTS_ROUTE, tournaments::get_routes::<S>())
}

#[derive(Debug)]
pub struct SquireRouter<S> {
    router: HashMap<&'static str, Router<S>>,
}

impl<S> SquireRouter<S>
where
    S: 'static + Clone + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            router: Default::default(),
        }
    }

    pub fn extend<const N: usize>(mut self, url: Url<N>, new_router: Router<S>) -> Self {
        let router = if let Some(router) = self.router.remove(url.as_str()) {
            router.merge(new_router)
        } else {
            new_router
        };
        self.router.insert(url.as_str(), router);
        Self {
            router: self.router,
        }
    }

    pub fn into(self) -> Router<S> {
        let mut router = Router::new();
        for (base, sub) in self.router {
            router = router.nest(base, sub);
        }
        router
    }
}

pub async fn get_version<S>(State(state): State<S>) -> ServerVersionResponse
where
    S: ServerState,
{
    ServerVersionResponse::new(state.get_version())
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
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
        Ok(Default::default())
        /*
        println!("Loading Cookies from parts...");
        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => StatusCode::FORBIDDEN,
                    _ => panic!("unexpected error getting Cookie header(s): {e}"),
                },
                _ => panic!("unexpected error getting cookies: {e}"),
            })?;

        println!("Looking for correct cookie:\n{cookies:?}");
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(StatusCode::FORBIDDEN)?;

        println!("Loading Session...");
        let session = state
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(StatusCode::FORBIDDEN)?;
        println!("Session loaded successfully!");

        session.get("user").ok_or(StatusCode::FORBIDDEN)
        */
    }
}

impl<S> Default for SquireRouter<S>
where
    S: 'static + Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl Default for User {
    fn default() -> Self {
        let id: SquireAccountId = Uuid::new_v4().into();
        let user_name = format!("Tester {id}");
        let display_name = format!("Tester {id}");
        let account = SquireAccount {
            user_name,
            display_name,
            id,
            gamer_tags: HashMap::new(),
            permissions: SharingPermissions::default(),
        };
        Self { account }
    }
}
