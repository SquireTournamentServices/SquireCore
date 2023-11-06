use std::time::Duration;

use futures::FutureExt;
use instant::Instant;
use send_wrapper::SendWrapper;

use super::{SendableFuture, Sleep};

/* ------ Send workarounds ------ */

pub trait Sendable: 'static {}

impl<T> Sendable for T where T: 'static {}

pub type SendableWrapper<T> = SendWrapper<T>;

/* ------ General Utils ------ */
/// Spawns a future that will execute in the background of the current thread. WASM bindgen's
/// `spawn_local` is used for this as tokio is caused problems in the browswer.
pub fn spawn_task<F, T>(fut: F)
where
    F: SendableFuture<Output = T>,
    T: Sendable,
{
    wasm_bindgen_futures::spawn_local(fut.map(drop));
}

/// Creates a future that will perform a non-blocking sleep
pub fn sleep(dur: Duration) -> Sleep {
    Sleep(Box::pin(gloo_timers::future::sleep(dur)))
}

/// Creates a future that will perform a non-blocking sleep
pub fn sleep_until(deadline: Instant) -> Sleep {
    Sleep(Box::pin(gloo_timers::future::sleep(
        deadline - Instant::now(),
    )))
}

pub fn log(_msg: &str) {
    //web_sys::console::log_1(&msg.into());
}

#[cfg(feature = "client")]
pub use client::*;

#[cfg(feature = "client")]
mod client {
    use std::{
        fmt::Debug,
        pin::Pin,
        task::{Context, Poll},
    };

    use futures::{Sink, Stream};
    use gloo_net::websocket::{
        futures::WebSocket as GlooSocket, Message as GlooMessage, WebSocketError as GlooError,
    };
    use reqwest::Response;
    use send_wrapper::SendWrapper;

    use crate::{
        client::error::{ClientError, ClientResult},
        compat::{WebsocketError, WebsocketMessage, WebsocketResult},
    };

    /* ------ Session ------ */

    /// A structure that the client uses to track its current session with the backend. A session
    /// represents both an active session and a yet-to-be-session.
    #[derive(Debug, Default, Clone)]
    pub struct Session {
        session: Option<()>,
    }

    impl Session {
        /// From a auth response from the backend, create and load the session as needed
        pub fn load_from_resp(&mut self, _resp: &Response) -> ClientResult<()> {
            // TODO: This is really all that we can do because of the browser?
            self.session = Some(());
            Ok(())
        }

        /// Create the session as a string in order to send a request
        pub fn cred_string(&self) -> ClientResult<String> {
            self.session
                .as_ref()
                .map(|_| String::new())
                .ok_or(ClientError::NotLoggedIn)
        }
    }

    /* ------ Websockets ------ */

    /// A wrapper around a `gloo_net` `WebSocket`. The "GlooSocket" is wrapped in a `SendWrapper`. This
    /// make the WASM websocket type `Send` like the native version but with the drawback that using it
    /// in another thread will cause a panic. This is a safe tradeoff since the WASM app only runs in a
    /// single thread in the browser.
    pub struct Websocket(SendWrapper<GlooSocket>);

    impl Debug for Websocket {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Websocket(..)")
        }
    }

    impl Websocket {
        /// Takes a URL string and attempts to connect to the backend at that URL. Because of
        /// compatability reason between the native and WASM Websockets, the request that is sent needs
        /// to be a simple get request.
        pub async fn new(url: &str) -> Result<Self, ()> {
            GlooSocket::open(url)
                .map(|sock| Websocket(SendWrapper::new(sock)))
                // TODO: Make this a real error...
                .map_err(|err| panic!("{err}"))
        }
    }

    impl Stream for Websocket {
        type Item = WebsocketResult;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Pin::new(&mut *self.0)
                .poll_next(cx)
                .map_err(Into::into)
                .map_ok(Into::into)
        }
    }

    impl Sink<WebsocketMessage> for Websocket {
        type Error = WebsocketError;

        fn poll_ready(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut *self.0).poll_ready(cx).map_err(Into::into)
        }

        fn start_send(mut self: Pin<&mut Self>, item: WebsocketMessage) -> Result<(), Self::Error> {
            Pin::new(&mut *self.0)
                .start_send(item.into())
                .map_err(Into::into)
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut *self.0).poll_flush(cx).map_err(Into::into)
        }

        fn poll_close(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut *self.0).poll_close(cx).map_err(Into::into)
        }
    }

    impl From<GlooError> for WebsocketError {
        fn from(_value: GlooError) -> Self {
            Self
        }
    }

    impl From<WebsocketMessage> for GlooMessage {
        fn from(value: WebsocketMessage) -> Self {
            match value {
                WebsocketMessage::Text(data) => Self::Text(data),
                WebsocketMessage::Bytes(data) => Self::Bytes(data),
            }
        }
    }

    impl From<GlooMessage> for WebsocketMessage {
        fn from(value: GlooMessage) -> Self {
            match value {
                GlooMessage::Text(data) => Self::Text(data),
                GlooMessage::Bytes(data) => Self::Bytes(data),
            }
        }
    }
}
