use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{Future, Sink, Stream};
use gloo_net::websocket::{
    futures::WebSocket as GlooSocket, Message as GlooMessage, WebSocketError as GlooError,
};
use reqwest::Response;

use super::{WebsocketError, WebsocketMessage, WebsocketResult};
use crate::client::error::{ClientError, ClientResult};

/* ------ General Utils ------ */

/// Spawns a future that will execute in the background of the current thread. WASM bindgen's
/// `spawn_local` is used for this as tokio is caused problems in the browswer.
pub fn spawn_task<F>(fut: F)
where
    F: 'static + Future<Output = ()>,
{
    wasm_bindgen_futures::spawn_local(fut);
}

/// Creates a future that will perform a non-blocking sleep
pub async fn rest(dur: Duration) {
    gloo_timers::future::sleep(dur).await;
}

pub fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

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

#[allow(missing_debug_implementations)]
pub struct Websocket(GlooSocket);

impl Websocket {
    /// Takes a URL string and attempts to connect to the backend at that URL. Because of
    /// compatability reason between the native and WASM Websockets, the request that is sent needs
    /// to be a simple get request.
    pub async fn new(url: &str) -> Result<Self, ()> {
        GlooSocket::open(url)
            .map(Websocket)
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

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(cx).map_err(Into::into)
    }

    fn start_send(mut self: Pin<&mut Self>, item: WebsocketMessage) -> Result<(), Self::Error> {
        Pin::new(&mut self.0)
            .start_send(item.into())
            .map_err(Into::into)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(cx).map_err(Into::into)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_close(cx).map_err(Into::into)
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
