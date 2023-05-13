//! This module contains the compatiablity layer to abstract over if the client is running natively
//! or in WASM. The goal of this module is to eliminate the use of this outside of this module:
//! ```
//! #[cfg(target_family = "wasm")]
//! ```
//!
//! By no means is this an exhuastive or future-proof module. Rather, the module just implements
//! wrappers for functionalities that are presently needed.

/// A common error return by the receiver half of an unbounded channel.
pub enum TryRecvError {
    Empty,
    Disconnected,
}

#[cfg(not(target_family = "wasm"))]
pub use native::*;

#[cfg(not(target_family = "wasm"))]
mod native {
    use std::{
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    };

    use cookie::Cookie;
    use futures::Future;
    use reqwest::Response;
    use tokio::sync::{mpsc, oneshot};

    use crate::{
        client::error::{ClientError, ClientResult},
        COOKIE_NAME,
    };

    use super::TryRecvError;

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

    #[derive(Debug)]
    pub struct UnboundedReceiver<T>(mpsc::UnboundedReceiver<T>);

    impl<T> UnboundedReceiver<T> {
        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            self.0.try_recv().map_err(Into::into)
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
        (UnboundedSender(send), UnboundedReceiver(recv))
    }

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
}

#[cfg(target_family = "wasm")]
pub use wasm::*;

#[cfg(target_family = "wasm")]
mod wasm {
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
        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            self.0.try_recv().map_err(Into::into)
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
}
