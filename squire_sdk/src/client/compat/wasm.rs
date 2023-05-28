use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use async_std::channel::{self, SendError, TrySendError};
use futures::{Future, Sink, Stream};
use gloo_net::websocket::{
    futures::WebSocket as GlooSocket, Message as GlooMessage, WebSocketError as GlooError,
};
use reqwest::Response;

use crate::client::error::{ClientError, ClientResult};

use super::{forget, TryRecvError, WebsocketError, WebsocketMessage, WebsocketResult};

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
    async_std::task::sleep(dur).await;
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
    pub fn load_from_resp(&mut self, resp: &Response) -> ClientResult<()> {
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

/* ------ Unbounded Channel ------ */

#[derive(Debug)]
pub struct UnboundedSender<T>(async_std::channel::Sender<T>);

impl<T> UnboundedSender<T> {
    pub fn send(&self, msg: T) -> Result<(), T> {
        self.0.try_send(msg).map_err(|e| match e {
            async_std::channel::TrySendError::Full(_) => {
                unreachable!("Unbounded sender was full")
            }
            async_std::channel::TrySendError::Closed(val) => val,
        })
    }
}

impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug)]
pub struct UnboundedReceiver<T>(async_std::channel::Receiver<T>);

impl<T> UnboundedReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await.ok()
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.0.try_recv().map_err(Into::into)
    }

    pub fn is_disconnected(&self) -> bool {
        self.0.is_closed()
    }
}

pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (send, recv) = async_std::channel::unbounded();
    (UnboundedSender(send), UnboundedReceiver(recv))
}

/* ------ Oneshot Channel ------ */

#[derive(Debug)]
pub struct OneshotSender<T>(async_std::channel::Sender<T>);

impl<T> OneshotSender<T> {
    pub fn send(self, msg: T) -> Result<(), T> {
        self.0.try_send(msg).map_err(|e| match e {
            async_std::channel::TrySendError::Full(_) => {
                unreachable!("One shot already sent...")
            }
            async_std::channel::TrySendError::Closed(val) => val,
        })
    }
}

#[derive(Debug)]
pub struct OneshotReceiver<T>(async_std::channel::Receiver<T>);

impl<T> OneshotReceiver<T> {
    pub async fn recv(self) -> Option<T> {
        self.0.recv().await.ok()
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.0.try_recv().map_err(Into::into)
    }
}

impl From<async_std::channel::TryRecvError> for TryRecvError {
    fn from(value: async_std::channel::TryRecvError) -> Self {
        match value {
            async_std::channel::TryRecvError::Empty => Self::Empty,
            async_std::channel::TryRecvError::Closed => Self::Disconnected,
        }
    }
}

pub fn oneshot<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    let (send, recv) = async_std::channel::bounded(1);
    (OneshotSender(send), OneshotReceiver(recv))
}

/* ------ Broadcast Channel ------ */

pub fn broadcast_channel<T: Clone>(capacity: usize) -> (Broadcaster<T>, Subscriber<T>) {
    let (send, recv) = channel::bounded(capacity);
    (Broadcaster(send), Subscriber(recv))
}

#[derive(Debug)]
pub struct Broadcaster<T>(async_std::channel::Sender<T>);

impl<T> Broadcaster<T> {
    pub fn send(&self, msg: T) -> Result<(), T> {
        self.0.try_send(msg).map_err(|err| match err {
            TrySendError::Full(val) | TrySendError::Closed(val) => val,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Subscriber<T>(async_std::channel::Receiver<T>);

impl<T: Clone> Subscriber<T> {
    pub async fn recv(&mut self) -> Result<T, ()> {
        self.0.recv().await.map_err(forget)
    }
}

/* ------ Websockets ------ */

pub struct Websocket(GlooSocket);

impl Websocket {
    /// Takes a URL string and attempts to connect to the backend at that URL. Because of
    /// compatability reason between the native and WASM Websockets, the request that is sent needs
    /// to be a simple get request.
    pub async fn new(url: &str) -> Result<Self, ()> {
        GlooSocket::open(url).map(Websocket).map_err(forget)
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
