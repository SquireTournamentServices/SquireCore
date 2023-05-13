//! This module contains the compatiablity layer to abstract over if the client is running natively
//! or in WASM. The goal of this module is to eliminate the use of this outside of this module:
//! ```
//! #[cfg(target_family = "wasm")]
//! ```
//!
//! By no means is this an exhuastive or future-proof module. Rather, the module just implements
//! wrappers for functionalities that are presently needed.

#[cfg(not(target_family = "wasm"))]
pub use native::*;

pub enum TryRecvError {
    Empty,
    Disconnected,
}

#[cfg(not(target_family = "wasm"))]
mod native {
    use std::{time::Duration, pin::Pin, task::{Context, Poll}};

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

    impl<T> Future for OneshotReceiver<T> {
        type Output = Result<T, TryRecvError>;

        fn poll(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Self::Output> {
            match self.0.try_recv() {
                Ok(_) => todo!(),
                Err(oneshot::error::TryRecvError::Empty) => todo!(),
                Err(oneshot::error::TryRecvError::Closed) => todo!(),
            }
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

    /// A structure that the client uses to track its current session with the backend. A session
    /// represents both an active session and a yet-to-be-session.
    #[derive(Debug, Default, Clone)]
    pub struct Session {
        session: Option<()>,
    }

    impl Session {
        /// From a auth response from the backend, create and load the session as needed
        pub fn load_from_resp(&mut self, resp: Response) -> ClientResult<()> {
            // TODO: This is really all that we can do because of the browser?
            self.session = Some(());
            Ok(())
        }

        /// Create the session as a string in order to send a request
        pub fn cred_string(&self) -> ClientResult<String> {
            self.session
                .as_ref()
                .map(String::new)
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
        gloo_timers::future::TimeoutFuture::new(dur.as_millis() as u32).await;
    }

    pub struct UnboundedSender<T> {}

    pub struct UnboundedReciever<T> {}

    pub fn unbounded_channel() -> (UnboundedSender, UnboundedReciever) {
        todo!()
    }

    pub struct OneshotSender<T> {}

    pub struct OneshotReciever<T> {}

    pub fn oneshot() -> (OneshotSender, OneshotReciever) {
        todo!()
    }
}
