use axum::response::{Html, Response};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use hyper::{body::Bytes, Body};

const INDEX_HTML: &str = include_str!("../../assets/index.html");
const APP_WASM: &[u8] = include_bytes!("../../assets/squire_web_bg.wasm");
const APP_JS: &str = include_str!("../../assets/squire_web.js");

pub async fn landing() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn get_wasm() -> Response<Body> {
    let bytes = Bytes::copy_from_slice(APP_WASM);
    let body: Body = bytes.into();

    Response::builder()
        .header(header::CONTENT_TYPE, "application/wasm")
        .body(body)
        .unwrap()
}

pub async fn get_js() -> (StatusCode, HeaderMap, &'static str) {
    let mut headers = HeaderMap::with_capacity(1);
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript;charset=utf-8"),
    );
    (StatusCode::OK, headers, APP_JS)
}
