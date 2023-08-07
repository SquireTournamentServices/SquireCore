use std::{convert::Infallible, future::Future, pin::Pin};

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, Response},
};
use http::{header::AUTHORIZATION, request::Parts, StatusCode};
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

/// The general session type that is returned by the SessionStore
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SquireSession {
    /// Credentials were no present
    NotLoggedIn,
    /// Credentials were present and corresponded to a logged-in user
    Active(SquireAccountId),
    /// Credentials were present but were past the expiry
    Expired(SquireAccountId),
    /// Credentials were present but corresponded to an unknown user
    UnknownUser,
}

/// The session of an active user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserSession(pub SquireAccountId);

/// A session type for APIs that accept either authenticated or unauthenticated users but that need
/// to distinquish between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnySession {
    Guest,
    User(SquireAccountId),
}

pub enum UserSessionError {
    /// Credentials were no present
    NotLoggedIn,
    /// Credentials were present but were past the expiry
    Expired,
    /// Credentials were present but corresponded to an unknown user
    UnknownUser,
}

impl SessionConvert for SquireSession {
    type Error = Infallible;

    fn convert(session: SquireSession) -> Result<Self, Self::Error> {
        Ok(session)
    }
}

impl SessionConvert for UserSession {
    type Error = UserSessionError;

    fn convert(session: SquireSession) -> Result<Self, Self::Error> {
        match session {
            SquireSession::Active(id) => Ok(Self(id)),
            SquireSession::NotLoggedIn => Err(UserSessionError::NotLoggedIn),
            SquireSession::Expired(_) => Err(UserSessionError::Expired),
            SquireSession::UnknownUser => Err(UserSessionError::UnknownUser),
        }
    }
}

impl IntoResponse for UserSessionError {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

pub trait SessionConvert: Sized {
    type Error: IntoResponse;

    fn convert(session: SquireSession) -> Result<Self, Self::Error>;
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
            let session = SquireSession::from_request_parts(parts, state)
                .await
                .unwrap();
            Se::convert(session).map(Session)
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SquireSession
where
    S: ServerState,
{
    type Rejection = Infallible;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let Some(header) = parts.headers.get(AUTHORIZATION) else {
                return Ok(Self::NotLoggedIn);
            };
            Ok(state.get_session(header.clone()).await)
        })
    }
}

