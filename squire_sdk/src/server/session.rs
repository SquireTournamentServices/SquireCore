use std::{convert::Infallible, future::Future, pin::Pin};

use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use hex::decode_to_slice;
use http::{header::AUTHORIZATION, request::Parts, HeaderMap, HeaderName, HeaderValue, StatusCode};
use squire_lib::identifiers::SquireAccountId;

use super::state::ServerState;

/* We will have two layers of session types.
 * The bottom layer is the session type that is returned by the session store. This is used to
 * communicate the abstract notion of a "session".
 *
 * The top layer consists of various API-specific session types. These sessions communiate various
 * business logic concepts. For example, the websocket API will be accessible to anyone that calls
 * it; however, we need to know if they are a known user or not in order to filter inbound
 * tournament updates. Contrast this with something like the GET account API, which requests that
 * you are logged in with an active session.
 *
 */

/// An extractor for a session type that can be converted from a `SquireSession`.
pub struct Session<T>(pub T);

/// The inner type used to represent all sessions
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct SessionToken(pub [u8; 32]);

impl From<[u8; 32]> for SessionToken {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

/// The general session type that is returned by the SessionStore
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SquireSession {
    /// Credentials were no present
    NotLoggedIn,
    /// Credentials were present but corresponded to an unknown user
    UnknownUser,
    /// The user has a guest session
    Guest(SessionToken),
    /// Credentials were present and corresponded to a logged-in user
    Active(SquireAccountId),
    /// Credentials were present but were past the expiry
    Expired(SquireAccountId),
}

/// The general session type that is returned by the SessionStore
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnyUser {
    /// The user has a guest session
    Guest(SessionToken),
    /// Credentials were present and corresponded to a logged-in user
    Active(SessionToken),
    /// Credentials were present but were past the expiry
    Expired(SessionToken),
}

impl SessionConvert for AnyUser {
    type Error = StatusCode;

    fn convert(token: SessionToken, session: SquireSession) -> Result<Self, Self::Error> {
        match session {
            SquireSession::Guest(token) => Ok(AnyUser::Guest(token)),
            SquireSession::Active(_id) => Ok(AnyUser::Active(token)),
            SquireSession::Expired(_id) => Ok(AnyUser::Expired(token)),
            SquireSession::NotLoggedIn | SquireSession::UnknownUser => {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }

    fn empty_session(_err: TokenParseError) -> Result<Self, Self::Error> {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// The session of an active user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserSession(pub SquireAccountId);

pub enum UserSessionError {
    /// Credentials were no present
    NotLoggedIn,
    /// Credentials were present but were past the expiry
    Expired,
    /// Credentials were present but corresponded to an unknown user
    UnknownUser,
    /// Credentials were present but corresponded to a guest
    Guest,
}

impl SessionConvert for SquireSession {
    type Error = Infallible;

    fn convert(_token: SessionToken, session: SquireSession) -> Result<Self, Self::Error> {
        Ok(session)
    }

    fn empty_session(_err: TokenParseError) -> Result<Self, Self::Error> {
        Ok(Self::NotLoggedIn)
    }
}

impl SessionConvert for UserSession {
    type Error = UserSessionError;

    fn convert(_token: SessionToken, session: SquireSession) -> Result<Self, Self::Error> {
        match session {
            SquireSession::Active(id) => Ok(Self(id)),
            SquireSession::NotLoggedIn => Err(UserSessionError::NotLoggedIn),
            SquireSession::Expired(_) => Err(UserSessionError::Expired),
            SquireSession::UnknownUser => Err(UserSessionError::UnknownUser),
            SquireSession::Guest(_) => Err(UserSessionError::Guest),
        }
    }

    fn empty_session(_err: TokenParseError) -> Result<Self, Self::Error> {
        Err(UserSessionError::NotLoggedIn)
    }
}

impl IntoResponse for UserSessionError {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

/// The trait is very similar to the `TryFrom` trait. It is used in conjuction with the `Session`
/// extractor to convert between `SquireSession`s and other session types. The `Sized` bound is
/// required since the `convert` method is a falliable constructor. The `Default` bound is used
/// when a session token can not be parsed from the headers.
pub trait SessionConvert: Sized {
    type Error: IntoResponse;

    /// A session token is present and needs to be converted.
    fn convert(token: SessionToken, session: SquireSession) -> Result<Self, Self::Error>;

    /// A session token is not present.
    fn empty_session(err: TokenParseError) -> Result<Self, Self::Error>;
}

impl<St, Se> FromRequestParts<St> for Session<Se>
where
    St: ServerState,
    Se: SessionConvert,
{
    type Rejection = Se::Error;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 St,
    ) -> Pin<Box<dyn 'async_trait + Send + Future<Output = Result<Self, Self::Rejection>>>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            match SessionToken::try_from(parts) {
                Ok(token) => {
                    let session = state.get_session(token.clone()).await;
                    Se::convert(token, session).map(Session)
                }
                Err(err) => Se::empty_session(err).map(Session),
            }
        })
    }
}

impl SessionToken {
    const HEADER_NAME: HeaderName = AUTHORIZATION;

    pub fn as_header(&self) -> (HeaderName, HeaderValue) {
        (
            AUTHORIZATION,
            HeaderValue::from_str(&hex::encode(self.0)).unwrap(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenParseError {
    NoAuthHeader,
    InvalidToken,
}

impl TryFrom<&mut Parts> for SessionToken {
    type Error = TokenParseError;

    fn try_from(parts: &mut Parts) -> Result<Self, Self::Error> {
        match parts.headers.get(Self::HEADER_NAME) {
            Some(header) => {
                let mut inner = [0; 32];
                let s = header.to_str().map_err(|_| TokenParseError::InvalidToken)?;
                decode_to_slice(s, &mut inner).map_err(|_| TokenParseError::InvalidToken)?;
                Ok(Self(inner))
            }
            None => Err(TokenParseError::NoAuthHeader),
        }
    }
}

impl TryFrom<&HeaderMap<HeaderValue>> for SessionToken {
    type Error = TokenParseError;

    fn try_from(headers: &HeaderMap) -> Result<Self, Self::Error> {
        match headers.get(Self::HEADER_NAME) {
            Some(header) => {
                let mut inner = [0; 32];
                let s = header.to_str().map_err(|_| TokenParseError::InvalidToken)?;
                decode_to_slice(s, &mut inner).map_err(|_| TokenParseError::InvalidToken)?;
                Ok(Self(inner))
            }
            None => Err(TokenParseError::NoAuthHeader),
        }
    }
}

impl IntoResponseParts for SessionToken {
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let (name, value) = self.as_header();
        let _ = res.headers_mut().insert(name, value);
        Ok(res)
    }
}

impl IntoResponse for SessionToken {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::with_capacity(1);
        let (name, value) = self.as_header();
        let _ = headers.insert(name, value);
        headers.into_response()
    }
}
