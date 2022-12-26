use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router, TypedHeader,
};
use headers::HeaderMap;
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    StatusCode,
};

use crate::{
    accounts::*,
    model::{
        accounts::{OrganizationAccount, SquireAccount},
        identifiers::{OrganizationAccountId as OrgId, SquireAccountId},
    },
    server::{AppState, User},
};

pub static COOKIE_NAME: &str = "SESSION";

pub fn get_routes() -> Router<AppState> {
    Router::new()
        .route("/verify", post(login))
        .route("/logout", post(logout))
}

pub async fn login(
    State(store): State<AppState>,
    Json(data): Json<LoginRequest>,
) -> impl IntoResponse {
    let account = USERS_MAP.get().unwrap().get(&data.id).unwrap().clone();
    let user = User { account };

    // Create a new session filled with user data
    let mut session = Session::new();
    session.insert("user", &user).unwrap();

    // Store session and get corresponding cookie
    let cookie = store.store_session(session).await.unwrap().unwrap();

    // Build the cookie
    let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; Path=/");

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, LoginResponse::new(Some(user.account)))
}

pub async fn logout(
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    State(store): State<AppState>,
) -> Result<StatusCode, Redirect> {
    let cookie = cookies.get(COOKIE_NAME).unwrap();
    let session = store
        .load_session(cookie.to_string())
        .await
        .unwrap()
        .ok_or(Redirect::to("/"))?;

    store.destroy_session(session).await.unwrap();

    Ok(StatusCode::OK)
}
