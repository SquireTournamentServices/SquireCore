use std::{net::SocketAddr, thread, time::Duration};

use async_session::MemoryStore;
use headers::HeaderValue;
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    StatusCode,
};

use reqwest::{cookie::Cookie, Client, Request, Response};
use serde::{Deserialize, Serialize};
use squire_sdk::{
    accounts::{CreateAccountRequest, CreateAccountResponse, LoginRequest, SquireAccountId},
    model::tournament::TournamentPreset,
    tournaments::{CreateTournamentRequest, CreateTournamentResponse},
};

use crate::{create_router, AppState, COOKIE_NAME};

use super::init::ensure_startup;

fn register_account_request() -> CreateAccountRequest {
    CreateAccountRequest {
        user_name: "Test User".into(),
        display_name: "Test".into(),
    }
}

fn login_request(id: SquireAccountId) -> LoginRequest {
    LoginRequest { id }
}

fn create_tournament_request() -> CreateTournamentRequest {
    CreateTournamentRequest {
        name: "Test".into(),
        preset: TournamentPreset::Swiss,
        format: "Pioneer".into(),
    }
}

async fn send_json_request<B>(client: &Client, path: &'static str, body: B) -> Response
where
    B: Serialize,
{
    client
        .post(format!("http://127.0.0.1:8000/api/v1/{path}"))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await
        .unwrap()
}

async fn send_json_request_with_cookie<B>(
    client: &Client,
    path: &'static str,
    cookie: Cookie<'static>,
    body: B,
) -> Response
where
    B: Serialize,
{
    client
        .post(format!("http://127.0.0.1:8000/api/v1/{path}"))
        .header(CONTENT_TYPE, "application/json")
        .header(SET_COOKIE, cookie.value().parse::<HeaderValue>().unwrap())
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn create_tournament_requires_login() {
    ensure_startup().await;
    println!("Server is running");

    let client = Client::new();
    let request = create_tournament_request();
    let resp = send_json_request(&client, "tournaments/create", &request).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_tournament() {
    ensure_startup().await;
    println!("Server is running");

    println!("Registering...");
    let client = Client::new();
    let request = register_account_request();
    let resp = send_json_request(&client, "register", &request).await;

    println!("Registered!!");
    assert_eq!(resp.status(), StatusCode::OK);

    tokio::time::sleep(Duration::from_millis(10)).await;

    println!("Logging in...");
    let account: CreateAccountResponse = resp.json().await.unwrap();
    let request = login_request(account.0.id);
    let resp = send_json_request(&client, "login", &request).await;
    let cookie = resp.cookies().find(|c| c.name() == COOKIE_NAME).unwrap();

    println!("Logged in!!");
    assert_eq!(resp.status(), StatusCode::OK);

    tokio::time::sleep(Duration::from_millis(1)).await;
}
