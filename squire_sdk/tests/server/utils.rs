use axum::{body::HttpBody, http::Request, response::Response};
use headers::HeaderValue;
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    Method,
};
use hyper::Body;
use serde::{de::DeserializeOwned, Serialize};
use tower::Service;

/*
use super::init::get_app;
pub use crate::utils::*;

pub async fn send_request(req: Request<Body>) -> Response {
    get_app().await.call(req).await.unwrap()
}

pub async fn extract_json_body<T>(resp: Response) -> T
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

pub fn get_cookies(resp: &Response) -> Vec<&HeaderValue> {
    resp.headers()
        .iter()
        .filter_map(|(name, val)| (name == SET_COOKIE).then_some(val))
        .collect()
}

pub fn create_request<B>(path: &str, body: B) -> Request<Body>
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
*/
