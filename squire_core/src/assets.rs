use axum::{
    response::{Html, Response},
    routing::get,
    Router,
};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use hyper::{body::Bytes, Body};

use crate::state::AppState;

const INDEX_HTML: &str = include_str!("../../assets/index.html");
const APP_WASM: &[u8] = include_bytes!("../../assets/squire_web_bg.wasm.gz");
const APP_JS: &str = include_str!("../../assets/squire_web.js");

pub fn inject_ui(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/", get(landing))
        .route("/squire_web_bg.wasm", get(get_wasm))
        .route("/squire_web.js", get(get_js))
        .fallback(landing)
}

pub async fn landing() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn get_wasm() -> (HeaderMap, &'static [u8]) {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip")); // Unzips the compressed file
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/wasm"),
    );
    (headers, APP_WASM)
}

pub async fn get_js() -> (StatusCode, HeaderMap, &'static str) {
    let mut headers = HeaderMap::with_capacity(1);
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript;charset=utf-8"),
    );
    (StatusCode::OK, headers, APP_JS)
}
