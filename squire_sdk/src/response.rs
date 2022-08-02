#[cfg(feature = "rocket")]
use std::io::Cursor;

#[cfg(feature = "rocket")]
use rocket::{
    response::{Responder, Result as RResult},
    Response,
};

#[cfg(feature = "rocket")]
use serde::Serialize;
use serde::Deserialize;

#[cfg(feature = "rocket")]
use crate::error::SERIALIZER_ERROR;

#[derive(Deserialize)]
pub struct SquireResponse<T>(pub T);

impl<T> SquireResponse<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

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
