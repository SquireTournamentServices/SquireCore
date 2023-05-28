use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use cookie::Cookie;
use futures::{Future, Sink, Stream};
use reqwest::Response;
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc, oneshot},
};
use tokio_tungstenite::{
    tungstenite::Error as TungsError, tungstenite::Message as TungsMessage, MaybeTlsStream,
    WebSocketStream,
};

use crate::{
    client::error::{ClientError, ClientResult},
    COOKIE_NAME,
};

use super::{TryRecvError, WebsocketError, WebsocketMessage, WebsocketResult, forget};

/* ------ General Utils ------ */

/// Spawns a future that will execute. The future must return nothing for compatability with the
/// WASM version.
pub fn spawn_task<F>(fut: F)
where
    F: 'static + Send + Future<Output = ()>,
{
    tokio::spawn(fut);
}

/// Creates a future that will perform a non-blocking sleep
pub async fn rest(dur: Duration) {
    tokio::time::sleep(dur).await;
}

/* ------ Session ------ */

/// A structure that the client uses to track its current session with the backend. A session
/// represents both an active session and a yet-to-be-session.
#[derive(Debug, Default, Clone)]
pub struct Session {
    cookie: Option<Cookie<'static>>,
}

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

/* ------ Unbounded Channel ------ */

#[derive(Debug)]
pub struct UnboundedSender<T>(mpsc::UnboundedSender<T>);

impl<T> UnboundedSender<T> {
    pub fn send(&self, msg: T) -> Result<(), T> {
        self.0.send(msg).map_err(|e| e.0)
    }
}

impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// A wrapper around the channel and a flag that tracks if the channel has been disconnected.
#[derive(Debug)]
pub struct UnboundedReceiver<T>(mpsc::UnboundedReceiver<T>, bool);

impl<T> UnboundedReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        let digest = self.0.recv().await;
        self.1 = digest.is_none();
        digest
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let digest: Result<_, TryRecvError> = self.0.try_recv().map_err(Into::into);
        self.1 = match digest.as_ref() {
            Ok(_) => false,
            Err(e) => e.is_disconnected(),
        };
        digest
    }

    pub fn is_disconnected(&self) -> bool {
        self.1
    }
}

impl From<mpsc::error::TryRecvError> for TryRecvError {
    fn from(value: mpsc::error::TryRecvError) -> Self {
        match value {
            mpsc::error::TryRecvError::Empty => Self::Empty,
            mpsc::error::TryRecvError::Disconnected => Self::Disconnected,
        }
    }
}

impl From<oneshot::error::TryRecvError> for TryRecvError {
    fn from(value: oneshot::error::TryRecvError) -> Self {
        match value {
            oneshot::error::TryRecvError::Empty => Self::Empty,
            oneshot::error::TryRecvError::Closed => Self::Disconnected,
        }
    }
}

pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (send, recv) = mpsc::unbounded_channel();
    (UnboundedSender(send), UnboundedReceiver(recv, false))
}

/* ------ Oneshot Channel ------ */

#[derive(Debug)]
pub struct OneshotSender<T>(oneshot::Sender<T>);

impl<T> OneshotSender<T> {
    pub fn send(self, msg: T) -> Result<(), T> {
        self.0.send(msg)
    }
}

#[derive(Debug)]
pub struct OneshotReceiver<T>(oneshot::Receiver<T>);

impl<T> OneshotReceiver<T> {
    pub async fn recv(self) -> Option<T> {
        self.0.await.ok()
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.0.try_recv().map_err(Into::into)
    }
}

pub fn oneshot<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    let (send, recv) = oneshot::channel();
    (OneshotSender(send), OneshotReceiver(recv))
}

/* ------ Broadcast Channel ------ */

pub fn broadcast_channel<T: Clone>(capacity: usize) -> (Broadcaster<T>, Subscriber<T>) {
    let (send, recv) = broadcast::channel(capacity);
    (Broadcaster(send), Subscriber(recv))
}

#[derive(Debug)]
pub struct Broadcaster<T>(broadcast::Sender<T>);

impl<T> Broadcaster<T> {
    pub fn send(&self, msg: T) -> Result<(), T> {
        self.0.send(msg).map(forget).map_err(|err| err.0)
    }
}

#[derive(Debug)]
pub struct Subscriber<T>(broadcast::Receiver<T>);

impl<T: Clone> Subscriber<T> {
    pub async fn recv(&mut self) -> Result<T, ()> {
        self.0.recv().await.map_err(forget)
    }
}

impl<T: Clone> Clone for Subscriber<T> {
    fn clone(&self) -> Self {
        Self(self.0.resubscribe())
    }
}

/* ------ Websockets ------ */

pub struct Websocket(WebSocketStream<MaybeTlsStream<TcpStream>>);

impl Websocket {
    /// Takes a URL string and attempts to connect to the backend at that URL. Because of
    /// compatability reason between the native and WASM Websockets, the request that is sent needs
    /// to be a simple get request.
    pub async fn new(url: &str) -> Result<Self, ()> {
        tokio_tungstenite::connect_async(url).await.map(|(ws, _)| Websocket(ws)).map_err(forget)
    }
}

impl Stream for Websocket {
    type Item = WebsocketResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx).map_ok(Into::into).map_err(Into::into)
    }
}

impl Sink<WebsocketMessage> for Websocket {
    type Error = WebsocketError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(cx).map_err(Into::into)
    }

    fn start_send(mut self: Pin<&mut Self>, item: WebsocketMessage) -> Result<(), Self::Error> {
        Pin::new(&mut self.0).start_send(item.into()).map_err(Into::into)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(cx).map_err(Into::into)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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
