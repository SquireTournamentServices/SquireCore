use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use cookie::Cookie;
use futures::Future;
use reqwest::Response;
use tokio::{sync::{broadcast, mpsc, oneshot}, net::TcpStream};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

use crate::{
    client::error::{ClientError, ClientResult},
    COOKIE_NAME,
};

use super::TryRecvError;

fn forget<T>(_: T) { }

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
