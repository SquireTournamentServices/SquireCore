use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{Future, Stream};
use instant::Instant;
use pin_project::pin_project;

use super::{SendableFuture, Sleep};

/* ------ Send workarounds ------ */

pub trait Sendable: 'static + Send {}

impl<T> Sendable for T where T: 'static + Send {}

#[pin_project]
pub struct SendableWrapper<T>(#[pin] T);

impl<T> SendableWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn take(self) -> T {
        let Self(inner) = self;
        inner
    }
}

impl<T: Send> Deref for SendableWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Send> DerefMut for SendableWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Future> Future for SendableWrapper<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().0.poll(cx)
    }
}

impl<T: Stream> Stream for SendableWrapper<T> {
    type Item = T::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().0.poll_next(cx)
    }
}

impl<T: Clone> Clone for SendableWrapper<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/* ------ General Utils ------ */

/// Spawns a future that will execute. The future must return nothing for compatability with the
/// WASM version.
pub fn spawn_task<F, T>(fut: F)
where
    F: SendableFuture<Output = T>,
    T: Sendable,
{
    drop(tokio::spawn(fut));
}

/// Creates a future that will perform a non-blocking sleep
pub fn sleep(dur: Duration) -> Sleep {
    Sleep(Box::pin(tokio::time::sleep(dur)))
}

/// Creates a future that will perform a non-blocking sleep
pub fn sleep_until(deadline: Instant) -> Sleep {
    Sleep(Box::pin(tokio::time::sleep_until(deadline.into())))
}

pub fn log(msg: &str) {
    println!("{msg}");
}

/* ------ Session ------ */
#[cfg(feature = "client")]
pub use client::*;

#[cfg(feature = "client")]
mod client {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use cookie::Cookie;
    use futures::{Sink, Stream};
    use reqwest::Response;
    use tokio::net::TcpStream;
    use tokio_tungstenite::{
        tungstenite::{Error as TungsError, Message as TungsMessage},
        MaybeTlsStream, WebSocketStream,
    };

    use crate::{
        client::error::{ClientError, ClientResult},
        compat::{WebsocketError, WebsocketMessage, WebsocketResult},
        COOKIE_NAME,
    };

    /// A structure that the client uses to track its current session with the backend. A session
    /// represents both an active session and a yet-to-be-session.
    #[cfg(feature = "client")]
    #[derive(Debug, Default, Clone)]
    pub struct Session {
        cookie: Option<Cookie<'static>>,
    }

    #[cfg(feature = "client")]
    impl Session {
        /// From a auth response from the backend, create and load the session as needed
        pub fn load_from_resp(&mut self, resp: &Response) -> ClientResult<()> {
            let session = resp
                .cookies()
                .find(|c| c.name() == COOKIE_NAME)
                .ok_or(ClientError::LogInFailed)?;
            let cookie = Cookie::build(COOKIE_NAME, session.value().to_string()).finish();
            self.cookie = Some(cookie);
            Ok(())
        }

        /// Create the session as a string in order to send a request
        pub fn cred_string(&self) -> ClientResult<String> {
            self.cookie
                .as_ref()
                .map(ToString::to_string)
                .ok_or(ClientError::NotLoggedIn)
        }
    }

    /* ------ Websockets ------ */

    #[derive(Debug)]
    pub struct Websocket(WebSocketStream<MaybeTlsStream<TcpStream>>);

    impl Websocket {
        /// Takes a URL string and attempts to connect to the backend at that URL. Because of
        /// compatability reason between the native and WASM Websockets, the request that is sent needs
        /// to be a simple get request.
        pub async fn new(url: &str) -> Result<Self, ()> {
            tokio_tungstenite::connect_async(url)
                .await
                .map(|(ws, _)| Websocket(ws))
                .map_err(|err| panic!("{err}"))
        }
    }

    impl Stream for Websocket {
        type Item = WebsocketResult;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Pin::new(&mut self.0)
                .poll_next(cx)
                .map_ok(Into::into)
                .map_err(Into::into)
        }
    }

    impl Sink<WebsocketMessage> for Websocket {
        type Error = WebsocketError;

        fn poll_ready(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_ready(cx).map_err(Into::into)
        }

        fn start_send(mut self: Pin<&mut Self>, item: WebsocketMessage) -> Result<(), Self::Error> {
            Pin::new(&mut self.0)
                .start_send(item.into())
                .map_err(Into::into)
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_flush(cx).map_err(Into::into)
        }

        fn poll_close(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_close(cx).map_err(Into::into)
        }
    }

    impl From<TungsError> for WebsocketError {
        fn from(_value: TungsError) -> Self {
            Self
        }
    }

    impl From<WebsocketMessage> for TungsMessage {
        fn from(value: WebsocketMessage) -> Self {
            match value {
                WebsocketMessage::Text(data) => Self::Text(data),
                WebsocketMessage::Bytes(data) => Self::Binary(data),
            }
        }
    }

    impl From<TungsMessage> for WebsocketMessage {
        fn from(value: TungsMessage) -> Self {
            match value {
                TungsMessage::Text(data) => Self::Text(data),
                TungsMessage::Binary(data) => Self::Bytes(data),
                TungsMessage::Ping(_)
                | TungsMessage::Pong(_)
                | TungsMessage::Close(_)
                | TungsMessage::Frame(_) => unreachable!("server sent invalid message"),
            }
        }
    }
}
