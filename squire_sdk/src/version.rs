use serde::{Serialize, Deserialize};

use crate::response::SquireResponse;

/// The response type for getting the server's version and mode
pub type ServerVersionResponse = SquireResponse<Version>;

/// The version of the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// The server's version number
    pub version: String,
    /// The mode the server is running in
    pub mode: ServerMode,
}

/// The mode that the server is running in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMode {
    /// The server only supports the basic API
    Basic,
    /// The server extends the basic API in some way
    Extended,
}
