use std::{fmt::Display, str::FromStr};

use hex::decode_to_slice;
use http::{header::AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use squire_lib::identifiers::SquireAccountId;

/// The inner type used to represent all sessions
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionToken(pub [u8; 32]);

impl From<[u8; 32]> for SessionToken {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenParseError {
    NoAuthHeader,
    InvalidToken,
}

impl SessionToken {
    pub const HEADER_NAME: HeaderName = AUTHORIZATION;

    pub fn as_header(&self) -> (HeaderName, HeaderValue) {
        (
            Self::HEADER_NAME,
            HeaderValue::from_str(&hex::encode(self.0)).unwrap(),
        )
    }

    pub fn as_raw_header(&self) -> (&'static str, String) {
        (Self::HEADER_NAME.as_str(), hex::encode(self.0))
    }
}

impl FromStr for SessionToken {
    type Err = TokenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut inner = [0; 32];
        decode_to_slice(s, &mut inner).map_err(|_| TokenParseError::InvalidToken)?;
        Ok(Self(inner))
    }
}

impl TryFrom<&HeaderMap<HeaderValue>> for SessionToken {
    type Error = TokenParseError;

    fn try_from(headers: &HeaderMap) -> Result<Self, Self::Error> {
        match headers.get(Self::HEADER_NAME).and_then(|h| h.to_str().ok()) {
            Some(header) => header.parse(),
            None => Err(TokenParseError::NoAuthHeader),
        }
    }
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = hex::encode(self.0);
        write!(f, "{data}")
    }
}

/// A user session for users that have an active session. Its primary usecase is for filtering
/// inbound websocket messages.
///
/// TODO: This type should also receive updates about the session so that such updates can be
/// communicated throughout the system.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AuthUser {
    Guest(SessionToken),
    User(SquireAccountId),
}
