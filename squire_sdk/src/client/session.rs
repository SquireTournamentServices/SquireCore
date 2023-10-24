use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;
use instant::{Duration, Instant};
use pin_project::pin_project;
use squire_lib::{accounts::SquireAccount, identifiers::SquireAccountId};
use tokio::sync::watch::{channel, Receiver as Watcher, Sender as Broadcaster};

use crate::compat::{sleep_until, Sleep};

const SESSION_DEADLINE: Duration = Duration::from_secs(604_800);

#[derive(Debug, Default)]
#[pin_project(project = ExpiryProj)]
pub enum Expiry {
    #[default]
    NoAuth,
    Deadline(#[pin] Sleep),
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum SessionInfo {
    /// No information is known about the user
    #[default]
    Unknown,
    /// The user has provided account information, but a session is not known
    User(SquireAccount),
    /// The user has started a guest session with the server
    Guest,
    /// The user has provided account information and has authenticated with the server
    AuthUser(SquireAccount),
}

#[derive(Debug)]
pub struct SessionBroadcaster {
    send: Broadcaster<SessionInfo>,
    // If the session is `AuthUser` or `Guest`, we need to track the expiry time of the session.
    // The held time is the time when the session will expire
    expiry: Option<Instant>,
}

impl Default for SessionBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionBroadcaster {
    pub fn new() -> Self {
        let (send, _) = channel(SessionInfo::Unknown);
        Self { send, expiry: None }
    }

    pub fn new_with_user(user: SquireAccount) -> Self {
        let (send, _) = channel(SessionInfo::User(user));
        Self { send, expiry: None }
    }

    pub fn subscribe(&self) -> SessionWatcher {
        let recv = self.send.subscribe();
        SessionWatcher::new(recv)
    }

    pub fn expiry(&self) -> Expiry {
        match self.expiry {
            Some(deadline) => Expiry::Deadline(sleep_until(deadline)),
            None => Expiry::NoAuth,
        }
    }

    pub fn user_auth(&mut self, user: SquireAccount) {
        self.expiry = Some(Instant::now() + SESSION_DEADLINE);
        self.send.send_modify(move |s| {
            *s = SessionInfo::AuthUser(user);
        });
    }

    pub fn guest_auth(&mut self) {
        self.expiry = Some(Instant::now() + SESSION_DEADLINE);
        self.send.send_modify(|s| {
            *s = SessionInfo::Guest;
        });
    }

    pub fn session_info(&self) -> SessionInfo {
        self.send.borrow().clone()
    }
}

impl Future for Expiry {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            ExpiryProj::NoAuth => Poll::Pending,
            ExpiryProj::Deadline(deadline) => deadline.poll(cx),
        }
    }
}

#[derive(Debug)]
pub struct SessionWatcher {
    recv: Watcher<SessionInfo>,
}

impl SessionWatcher {
    pub fn new(recv: Watcher<SessionInfo>) -> Self {
        Self { recv }
    }

    pub fn session_query<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&SessionInfo) -> O,
    {
        f(&self.recv.borrow())
    }

    pub fn session_info(&self) -> SessionInfo {
        self.recv.borrow().clone()
    }

    pub fn get_squire_account_id(&self) -> Option<SquireAccountId> {
        self.session_info().get_user().map(|acc| acc.id)
    }
}

impl SessionInfo {
    pub fn get_user(&self) -> Option<SquireAccount> {
        match self {
            SessionInfo::Unknown | SessionInfo::Guest => None,
            SessionInfo::User(user) | SessionInfo::AuthUser(user) => Some(user.clone()),
        }
    }
}
