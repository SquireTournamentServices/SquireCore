#[cfg(feature = "rocket")]
use rocket::http::Status;

#[cfg(feature = "rocket")]
pub const SERIALIZER_ERROR: Status = Status { code: 69 };
