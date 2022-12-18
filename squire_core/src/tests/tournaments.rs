use std::{net::SocketAddr, rc::Rc, thread, time::Duration};

use async_session::MemoryStore;
use headers::HeaderValue;
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    Method, StatusCode,
};

use axum::{body::HttpBody, handler::Handler, http::Request, response::Response};
use hyper::Body;
use serde::{Deserialize, Serialize};
use tower::{Service, ServiceExt};

use squire_sdk::{
    accounts::{CreateAccountRequest, CreateAccountResponse, LoginRequest, SquireAccountId},
    model::tournament::TournamentPreset,
    tournaments::{CreateTournamentRequest, CreateTournamentResponse},
};

use crate::{create_router, AppState, COOKIE_NAME};

use super::init::get_app;

fn create_request<B>(path: &str, body: B) -> Request<Body>
where
    B: Serialize,
{
    Request::builder()
        .method(Method::POST)
        .uri(format!("http://127.0.0.1:8000/api/v1/{path}"))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&body).unwrap().into())
        .unwrap()
}

fn register_account_request() -> Request<Body> {
    let body = CreateAccountRequest {
        user_name: "Test User".into(),
        display_name: "Test".into(),
    };
    create_request("register", body)
}

fn login_request(id: SquireAccountId) -> Request<Body> {
    let body = LoginRequest { id };
    create_request("login", body)
}

fn create_tournament_request() -> Request<Body> {
    let body = CreateTournamentRequest {
        name: "Test".into(),
        preset: TournamentPreset::Swiss,
        format: "Pioneer".into(),
    };
    create_request("tournaments/create", body)
}

async fn send_request(req: Request<Body>) -> Response {
    get_app()
        .await
        .ready()
        .await
        .unwrap()
        .call(req)
        .await
        .unwrap()
}

#[tokio::test]
async fn create_tournament_requires_login() {
    let request = create_tournament_request();
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_tournament() {
    let request = register_account_request();
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let account: CreateAccountResponse = serde_json::from_str(
        &String::from_utf8(resp.into_body().data().await.unwrap().unwrap().to_vec()).unwrap(),
    ).unwrap();
    let request = login_request(account.0.id);
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::OK);
    //let cookie = resp.cookies().find(|c| c.name() == COOKIE_NAME).unwrap();
}
