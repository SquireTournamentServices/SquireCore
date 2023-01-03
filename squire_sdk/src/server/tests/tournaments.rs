use std::{net::SocketAddr, rc::Rc, thread, time::Duration};

use async_session::MemoryStore;
use headers::{Cookie, HeaderName, HeaderValue};
use http::{
    header::{self, CONTENT_TYPE, SET_COOKIE},
    Method, StatusCode,
};

use axum::{body::HttpBody, handler::Handler, http::Request, response::Response};
use hyper::Body;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tower::{Service, ServiceExt};

use crate::{
    accounts::{CreateAccountRequest, CreateAccountResponse, LoginRequest, SquireAccountId},
    model::tournament::TournamentPreset,
    server::{
        create_router,
        tests::{
            init::get_app,
            requests::{create_tournament_request, login_request, register_account_request},
            utils::*,
        },
        COOKIE_NAME,
    },
    tournaments::{CreateTournamentRequest, CreateTournamentResponse},
};

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

    let account: CreateAccountResponse = extract_json_body(resp).await;

    let request = login_request(account.0.id);
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let cookies = get_cookies(&resp);

    let mut request = create_tournament_request();
    request
        .headers_mut()
        .insert(header::COOKIE, cookies[0].clone());
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let tourn: CreateTournamentResponse = extract_json_body(resp).await;
}
