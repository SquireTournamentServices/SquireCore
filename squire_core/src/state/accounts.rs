use axum::response::IntoResponse;
use reqwest::Response;

pub enum LoginError {}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        todo!()
    }
}
