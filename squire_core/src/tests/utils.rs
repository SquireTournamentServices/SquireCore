/*
use axum::{body::HttpBody, http::Request, response::Response};
use headers::HeaderValue;
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    Method,
};
use serde::{de::DeserializeOwned, Serialize};
use squire_sdk::api::Url;
use tower::{Service, ServiceExt};

use crate::tests::init::get_app;

pub(crate) async fn send_request(req: Request<Body>) -> Response {
    get_app()
        .await
        .ready()
        .await
        .unwrap()
        .call(req)
        .await
        .unwrap()
}

pub(crate) async fn extract_json_body<T>(resp: Response) -> T
where
    T: DeserializeOwned,
{
    let mut body = resp.into_body();
    let mut buffer = Vec::new();
    while let Some(data) = body.data().await {
        buffer.extend(data.unwrap());
    }
    let data = String::from_utf8(buffer).expect("Could not form string from body data");
    serde_json::from_str(&data).expect("Could not deserialize body data")
}

pub(crate) fn get_cookies(resp: &Response) -> Vec<&HeaderValue> {
    resp.headers()
        .iter()
        .filter_map(|(name, val)| (name == SET_COOKIE).then_some(val))
        .collect()
}

pub(crate) fn create_request<B>(path: &Url<0>, body: B) -> Request<Body>
where
    B: Serialize,
{
    Request::builder()
        .method(Method::POST)
        .uri(format!("http://127.0.0.1:8000/{path}"))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&body).unwrap().into())
        .unwrap()
}
*/
