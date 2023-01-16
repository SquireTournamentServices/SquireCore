use std::sync::Arc;

use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    extract::{Path, State},
    handler::Handler,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router, TypedHeader,
};
use headers::HeaderMap;
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    StatusCode,
};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::{
    accounts::*,
    model::{
        accounts::{OrganizationAccount, SquireAccount},
        identifiers::{OrganizationAccountId as OrgId, SquireAccountId},
    },
    server::{state::ServerState, User, COOKIE_NAME},
};

pub fn get_routes<S>() -> Router<S>
where
    S: ServerState,
{
    Router::new()
        .route("/verify", post(post_verify::<S>).get(get_verify::<S>))
        .route("/logout", post(logout::<S>))
        .route("/load", post(load_user::<S>))
}

pub async fn get_verify<S>(user: User, State(state): State<S>) -> VerificationResponse
where
    S: ServerState,
{
    VerificationResponse::new(
        state
            .get_verification_data(&user)
            .ok_or(VerificationError::UnknownAccount),
    )
}

pub async fn post_verify<S>(
    State(state): State<S>,
    Json(data): Json<VerificationRequest>,
) -> (HeaderMap, VerificationResponse)
where
    S: ServerState,
{
    println!("Processing verification request");
    let user = match state
        .get_user(&data.account.id)
        .await
        .ok_or(VerificationError::UnknownAccount)
    {
        Ok(user) => user,
        Err(err) => return (HeaderMap::new(), VerificationResponse::new(Err(err))),
    };

    // Create a new session filled with user data
    let mut session = Session::new();
    session.insert("user", &user).unwrap();

    // Store session and get corresponding cookie
    let cookie = state.store_session(session).await.unwrap().unwrap();

    // Build the cookie
    let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; Path=/");

    let data = state.create_verification_data(&user).await;

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, VerificationResponse::new(Ok(data)))
}

pub async fn logout<S>(
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    State(store): State<S>,
) -> Result<StatusCode, Redirect>
where
    S: ServerState,
{
    let cookie = cookies.get(COOKIE_NAME).unwrap();
    let session = store
        .load_session(cookie.to_string())
        .await
        .unwrap()
        .ok_or(Redirect::to("/"))?;

    store.destroy_session(session).await.unwrap();

    Ok(StatusCode::OK)
}

pub async fn load_user<S>(State(state): State<S>, Json(user): Json<User>) -> StatusCode
where
    S: ServerState,
{
    state.load_user(user);
    StatusCode::ACCEPTED
}

impl VerificationData {
    fn new(key: String) -> VerificationData {
        VerificationData {
            confirmation: key.to_owned(),
            status: false,
        }
    }
}

pub fn generate_key() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

pub fn attempt_verification(account: SquireAccount) -> VerificationRequest {
    VerificationRequest { account }
}
