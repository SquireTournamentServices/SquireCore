use std::ops::{Deref, DerefMut};

#[cfg(feature = "axum")]
use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

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

impl<T> From<T> for SquireResponse<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for SquireResponse<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SquireResponse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "axum")]
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

/* TODO: Add back when feature is stable
// Example: Fn() -> Result<O, R>? => SquireResponse(Err(R))
impl<T, O, R> Try for SquireResponse<T>
where
    T: Try<Output = O, Residual = R> + FromResidual,
{
    type Output = O;
    type Residual = R;

    fn from_output(output: Self::Output) -> Self {
        Self::new(T::from_output(output))
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        self.0.branch()
    }
}

impl<T, O, R> FromResidual for SquireResponse<T>
where
    T: Try<Output = O, Residual = R> + FromResidual,
{
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        Self::new(T::from_residual(residual))
    }
}
*/
