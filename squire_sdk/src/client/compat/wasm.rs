use std::time::Duration;

use futures::Future;
use reqwest::Response;

use crate::client::error::{ClientError, ClientResult};

use super::TryRecvError;

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

/// Spawns a future that will execute in the background of the current thread. WASM bindgen's
/// `spawn_local` is used for this as tokio is caused problems in the browswer.
pub fn spawn_task<F>(fut: F)
where
    F: 'static + Future<Output = ()>,
{
    wasm_bindgen_futures::spawn_local(async { fut.await });
}

/// Creates a future that will perform a non-blocking sleep
pub async fn rest(dur: Duration) {
    async_std::task::sleep(dur).await;
}

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

pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (send, recv) = async_std::channel::unbounded();
    (UnboundedSender(send), UnboundedReceiver(recv))
}

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

pub fn broadcast_channel<T: Clone>(capacity: usize) -> (Broadcaster<T>, Subscriber<T>) {
    let (send, recv) = broadcast::channel(capacity);
    (Broadcaster(send), Subscriber(recv))
}

#[derive(Debug)]
pub struct Broadcaster<T>(async_std::channel::Receiver<T>);

impl<T> Broadcaster<T> {
    pub fn send(&self, msg: T) -> Result<(), T> {
        self.0.try_send(msg).map_err(|err| match err {
            TrySendError::Closed(val) | TrySendError::Full(val) => val,
        })
    }
}

#[derive(Debug)]
pub struct Subscriber<T>(async_std::channel::Sender<T>);

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
