#[cfg(feature = "rocket")]
use std::io::Cursor;

#[cfg(feature = "rocket")]
use rocket::{
    response::{Responder, Result as RResult},
    Response,
};

use serde::Deserialize;
#[cfg(feature = "rocket")]
use serde::Serialize;

#[cfg(feature = "rocket")]
use crate::error::SERIALIZER_ERROR;

#[derive(Debug, Deserialize)]
/// This is the base wrapper struct used to wrap SC response data. This prevents having to
/// reimplement the `Responder` trait for every new response type.
pub struct SquireResponse<T>(pub T);

impl<T> SquireResponse<T> {
    /// Creates a new `SquireResponse` object
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

// The `Responder` trait is only needed by the SC server.
#[cfg(feature = "rocket")]
impl<'r, T> Responder<'r, 'r> for SquireResponse<T>
where
    T: Serialize + Deserialize<'r>,
{
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> RResult<'r> {
        match serde_json::to_string(&self.0) {
            Err(_) => RResult::Err(SERIALIZER_ERROR),
            Ok(data) => {
                let resp = Response::build()
                    .sized_body(data.len(), Cursor::new(data))
                    .finalize();
                RResult::Ok(resp)
            }
        }
    }
}
