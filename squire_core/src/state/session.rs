//! To start, we will use a primative session store with a simple peristence model.
//!
//! A session token will consist of 256 bits (32 bytes) represented as a hex string in the session
//! cookie. The session store will be a simple map from thesse byte slices to session info,
//! including the account ID, expiration date, and eviction date. Sessions will be valid for 10
//! days with a 2 day eviction notice. Between expiration and eviction, a token will not be useable
//! except for reauth operations. At which point, a new token will be issued.
//!
//! Active sessions will be persisted in the database as tuples of (id, token, creation time).
//! When a session expires, it will be removed from the active sessions table and inserted into a
//! past sessions table. After the session has fully expired, the session will be removed
//! completely.
//!
//! Session creation happens via the `login` API. This consumes a user's credentials (username and
//! password). Assuming the provided password matches the stored password (and the user can be
//! found), a new session is created. The session and the user's account ID for returned to the
//! caller.
//!
//! A couple of notes, for a good user expirence, we will need three things:
//!  - Guest sessions
//!  - Session promotion
//!  - Links between sessions and WS connections (internal)
//!
//! Guest sessions are straightforward, we need a why to identify users as guests by providing them
//! with session tokens. On their own, these sessions don't do anything, but they enable the other
//! two features. Note that these sessions need to have an expiration date, but will not have an
//! account id associated with them.
//!
//! Session promotion occurs when a guest user logs in. This will either assoicate a session with
//! their account id or reissue a new session token and eliminate the old one.
//!
//! These two features combine (with some additional work) to make websocket connections seemless
//! should a user view at a tournament then log in. This promotion should upgrade what they can see
//! and do in the tournament while maintaining their WS connection. This will likely be done via a
//! broadcast channel to broadcast guest session promotions.
//!
//! Note that the work required to make guest session promotion create a better WS expirence is
//! also needed for reauth connections. If a user establishes a WS connection, the connection
//! should be termianted (or at least downgraded to be that of a guest) once it expires. Reauth
//! should reset the timer in the task that manages a tournament's WS connections.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use http::HeaderValue;
use squire_sdk::{model::identifiers::SquireAccountId, server::session::SquireSession};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::{channel as oneshot_channel, Receiver as OneshotReceiver, Sender as OneshotSender},
};

pub struct Tracker<T> {
    recv: OneshotReceiver<T>,
}

impl<T> Future for Tracker<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We unwrap here because if the task hangs up, it has panicked, which should not happen
        // unless something very bad has happened
        Pin::new(&mut self.recv).poll(cx).map(Result::unwrap)
    }
}

impl<T> Tracker<T> {
    fn new(recv: OneshotReceiver<T>) -> Self {
        Self { recv }
    }
}

#[derive(Debug, Clone)]
pub struct SessionStoreHandle {
    handle: UnboundedSender<SessionCommand>,
}

/// The type that is used internally to represent a session token.
pub type SessionInner = Vec<u8>;

pub enum SessionCommand {
    Create(OneshotSender<SessionInner>),
    Get(HeaderValue, OneshotSender<SquireSession>),
    Reauth(SquireAccountId, OneshotSender<SessionInner>),
    Delete(SquireAccountId, OneshotSender<bool>),
}

impl SessionStoreHandle {
    pub fn new() -> Self {
        let (send, recv) = unbounded_channel();
        let store = SessionStore::new(recv);
        tokio::spawn(store.run());
        Self { handle: send }
    }

    pub fn create(&self) -> Tracker<SessionInner> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(SessionCommand::Create(send));
        Tracker::new(recv)
    }

    pub fn get(&self, header: HeaderValue) -> Tracker<SquireSession> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(SessionCommand::Get(header, send));
        Tracker::new(recv)
    }

    pub fn reauth(&self, id: SquireAccountId) -> Tracker<SessionInner> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(SessionCommand::Reauth(id, send));
        Tracker::new(recv)
    }

    pub fn delete(&self, id: SquireAccountId) -> Tracker<bool> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(SessionCommand::Delete(id, send));
        Tracker::new(recv)
    }
}

struct SessionStore {
    inbound: UnboundedReceiver<SessionCommand>,
}

impl SessionStore {
    fn new(inbound: UnboundedReceiver<SessionCommand>) -> Self {
        Self { inbound }
    }

    async fn run(mut self) -> ! {
        loop {
            let _msg = self.inbound.recv().await.unwrap();
        }
    }
}
