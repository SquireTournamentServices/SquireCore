use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
/// This is the base wrapper struct used to wrap SC response data. This prevents having to
/// reimplement the `Responder` trait for every new response type.
pub struct SquireResponse<T>(pub T);

impl<T> SquireResponse<T> {
    /// Creates a new `SquireResponse` object
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<'r, T> IntoResponse for SquireResponse<T>
where
    T: Serialize + Deserialize<'r>,
{
    fn into_response(self) -> axum::response::Response {
        match serde_json::to_string(&self.0) {
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize data!!",
            )
                .into_response(),
            Ok(data) => data.into_response(),
        }
    }
}
