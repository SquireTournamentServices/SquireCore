#[cfg(feature = "rocket")]
use rocket::http::Status;

#[cfg(feature = "rocket")]
/// A shorthand used by SquireCore
pub const SERIALIZER_ERROR: Status = Status { code: 69 };
