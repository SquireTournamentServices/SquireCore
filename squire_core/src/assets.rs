#[allow(clippy::wildcard_imports)]
pub use frontend::*;

#[cfg(feature = "ignore-frontend")]
mod frontend {
    use axum::response::Html;

    pub async fn landing() -> Html<&'static str> {
        Html("Frontend not compiled...")
    }

    pub async fn get_wasm() {}

    pub async fn get_js() {}
}

#[cfg(all(feature = "ignore-frontend", not(debug_assertions)))]
compile_error!("In release mode, you must compile the frontend!");

#[cfg(not(feature = "ignore-frontend"))]
mod frontend {
    use axum::response::Html;
    use http::{header, HeaderMap, HeaderValue};

    const INDEX_HTML: &str = include_str!("../../assets/index.html");
    const APP_WASM: &'static [u8] = include_bytes!("../../assets/squire_web_bg.wasm.gz");
    const APP_JS: &str = include_str!("../../assets/squire_web.js");

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

    pub async fn get_js() -> (HeaderMap, &'static str) {
        let mut headers = HeaderMap::with_capacity(1);
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/javascript;charset=utf-8"),
        );
        (headers, APP_JS)
    }
}
