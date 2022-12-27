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

use super::state::ServerState;

pub static COOKIE_NAME: &str = "SESSION";

pub fn get_routes() -> Router<AppState> {
    Router::new()
        .route("/verify", post(post_verify).get(get_verify))
        .route("/logout", post(logout))
}

pub async fn get_verify(user: User, State(state): State<AppState>) -> VerificationResponse {
    VerificationResponse::new(
        state
            .get_verification_data(&user)
            .ok_or(VerificationError::UnknownAccount),
    )
}

pub async fn post_verify(
    State(state): State<AppState>,
    Json(data): Json<LoginRequest>,
) -> (HeaderMap, VerificationResponse) {
    let user = match state
        .get_user(&data.id)
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

    let data = VerificationData {
        confirmation: state.create_verification_data(&user).await,
        status: false,
    };

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, VerificationResponse::new(Ok(data)))
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
