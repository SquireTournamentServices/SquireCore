use std::collections::HashMap;

use async_session::async_trait;
use axum::{
    body::{Body, HttpBody},
    extract::{FromRequestParts, State},
    handler::Handler,
    http::StatusCode,
    Router,
};
use http::request::Parts;
use serde::{Deserialize, Serialize};
use squire_lib::{
    accounts::{SharingPermissions, SquireAccount},
    identifiers::SquireAccountId,
};
use uuid::Uuid;

use self::{gathering::init_gathering_hall, state::ServerState};
use crate::api::*;

pub mod gathering;
pub mod state;
pub mod tournaments;

pub fn create_router<S: ServerState>(state: S) -> SquireRouter<S, Body> {
    init_gathering_hall(state);
    get_routes::<S>().merge(tournaments::get_routes::<S>())
}

fn get_routes<S: ServerState>() -> SquireRouter<S> {
    SquireRouter::new().add_route::<0, GET, GetVersion, _, _>(get_version::<S>)
}

#[derive(Debug)]
pub struct SquireRouter<S, B = Body> {
    router: Router<S, B>,
}

impl<S, B> SquireRouter<S, B>
where
    S: ServerState,
    B: HttpBody + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            router: Default::default(),
        }
    }

    pub fn add_route<const N: usize, const M: u8, R, T, H>(self, handler: H) -> Self
    where
        R: RestRequest<N, M>,
        T: 'static,
        H: Handler<T, S, B>,
    {
        Self {
            router: self.router.route(R::ROUTE.as_str(), R::as_route(handler)),
        }
    }

    pub fn merge(self, Self { router }: Self) -> Self {
        Self {
            router: self.router.merge(router),
        }
    }

    pub fn into_router(self) -> Router<S, B> {
        let Self { router } = self;
        router
    }
}

pub async fn get_version<S: ServerState>(State(state): State<S>) -> ServerVersionResponse {
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

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Default::default())
        /*
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

        let session_cookie = cookies.get(COOKIE_NAME).ok_or(StatusCode::FORBIDDEN)?;

        let session = state
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(StatusCode::FORBIDDEN)?;

        session.get("user").ok_or(StatusCode::FORBIDDEN)
        */
    }
}

impl<S> Default for SquireRouter<S, Body>
where
    S: ServerState,
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
