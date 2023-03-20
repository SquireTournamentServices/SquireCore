use axum::response::{Response, Html};
use http::header;
use hyper::{body::Bytes, Body};

#[cfg(debug_assertions)]
pub async fn landing() -> Html<String> {
    Html(
        tokio::fs::read_to_string("../assets/index.html")
            .await
            .unwrap(),
    )
}

#[cfg(debug_assertions)]
pub async fn get_wasm() -> Response<Body> {
    let wasm = tokio::fs::read("../assets/squire_web_bg.wasm")
        .await
        .unwrap();
    let bytes = Bytes::copy_from_slice(&wasm);
    let body: Body = bytes.into();

    Response::builder()
        .header(header::CONTENT_TYPE, "application/wasm")
        .body(body)
        .unwrap()
}

#[cfg(not(debug_assertions))]
const INDEX_HTML: &str = include_str!("../../assets/index.html");
#[cfg(not(debug_assertions))]
const APP_WASM: &[u8] = include_bytes!("../../assets/squire_web_bg.wasm");
//#[cfg(not(debug_assertions))]
const APP_JS: &str = include_str!("../../assets/squire_web.js");

#[cfg(not(debug_assertions))]
pub async fn landing() -> Html<&'static str> {
    Html(INDEX_HTML)
}

#[cfg(not(debug_assertions))]
pub async fn get_wasm() -> Response<Body> {
    let bytes = Bytes::copy_from_slice(APP_WASM);
    let body: Body = bytes.into();

    Response::builder()
        .header(header::CONTENT_TYPE, "application/wasm")
        .body(body)
        .unwrap()
}

#[cfg(not(debug_assertions))]
pub async fn get_js() -> Response<String> {
    Response::builder()
        .header(header::CONTENT_TYPE, "application/javascript;charset=utf-8")
        .body(APP_JS.to_string())
        .unwrap()
}

#[cfg(debug_assertions)]
pub async fn get_js() -> Response<String> {
    let js = tokio::fs::read_to_string("../assets/squire_web.js")
        .await
        .unwrap();

    Response::builder()
        .header(header::CONTENT_TYPE, "application/javascript;charset=utf-8")
        .body(js)
        .unwrap()
}
