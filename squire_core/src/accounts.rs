use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    body::HttpBody,
    extract::State,
    response::{IntoResponse, Redirect},
    Json, TypedHeader,
};
use dashmap::DashMap;
use headers::HeaderMap;
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    Response, StatusCode,
};
use once_cell::sync::OnceCell;

use squire_sdk::{
    accounts::{LoginRequest, LoginResponse},
    model::{
        accounts::{OrganizationAccount, SquireAccount},
        identifiers::{AdminId, OrganizationAccountId as OrgId, SquireAccountId},
    },
};

use squire_sdk::{
    accounts::{
        CreateAccountRequest, CreateAccountResponse, GetAllUsersResponse, GetOrgResponse,
        GetUserPermissionsResponse, GetUserResponse, UpdateOrgAccountRequest,
        UpdateOrgAccountResponse, UpdateSquireAccountRequest, UpdateSquireAccountResponse,
    },
    Action,
};

use crate::User;

pub static USERS_MAP: OnceCell<DashMap<SquireAccountId, SquireAccount>> = OnceCell::new();
pub static ORGS_MAP: OnceCell<DashMap<OrgId, OrganizationAccount>> = OnceCell::new();
pub static COOKIE_NAME: &str = "SESSION";

pub fn init() {
    USERS_MAP.set(DashMap::new()).unwrap();
}

pub async fn register(Json(data): Json<CreateAccountRequest>) -> CreateAccountResponse {
    let account = SquireAccount::new(data.user_name, data.display_name);
    USERS_MAP.get().unwrap().insert(account.id, account.clone());
    CreateAccountResponse::new(account)
}

pub async fn login(
    State(store): State<MemoryStore>,
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
    let cookie = format!("{}={}; SameSite=Lax; Path=/", COOKIE_NAME, cookie);

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, LoginResponse::new(Some(user.account)))
}

#[axum::debug_handler]
pub async fn logout(
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    State(store): State<MemoryStore>,
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
