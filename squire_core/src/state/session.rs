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

use std::collections::HashSet;

use cycle_map::GroupMap;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use squire_sdk::{
    actor::*,
    api::SessionToken,
    model::identifiers::SquireAccountId,
    server::session::{AnyUser, SquireSession},
};
use tokio::sync::oneshot::Sender as OneshotSender;

#[derive(Debug, Clone)]
pub struct SessionStoreHandle {
    client: ActorClient<SessionStore>,
}

impl SessionStoreHandle {
    pub fn new() -> Self {
        let client = ActorClient::builder(SessionStore::new()).launch();
        Self { client }
    }

    pub fn create(&self, id: SquireAccountId) -> Tracker<SessionToken> {
        self.client.track(id)
    }

    pub fn guest(&self) -> Tracker<SessionToken> {
        self.client.track(())
    }

    pub fn get(&self, token: SessionToken) -> Tracker<SquireSession> {
        self.client.track(token)
    }

    pub fn reauth(&self, id: AnyUser) -> Tracker<SessionToken> {
        self.client.track(id)
    }

    pub fn delete(&self, id: AnyUser) -> Tracker<bool> {
        self.client.track(id)
    }
}

pub enum SessionCommand {
    Create(SquireAccountId, OneshotSender<SessionToken>),
    Guest(OneshotSender<SessionToken>),
    Get(SessionToken, OneshotSender<SquireSession>),
    Reauth(AnyUser, OneshotSender<SessionToken>),
    Delete(AnyUser, OneshotSender<bool>),
}

struct SessionStore {
    rng: StdRng,
    users: GroupMap<SessionToken, SquireAccountId>,
    guests: HashSet<SessionToken>,
}

#[async_trait]
impl ActorState for SessionStore {
    type Message = SessionCommand;

    async fn process(&mut self, _scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            SessionCommand::Create(id, send) => {
                let _ = send.send(self.create_session(id));
            }
            SessionCommand::Get(token, send) => {
                let _ = send.send(self.get_session(token));
            }
            SessionCommand::Reauth(id, send) => {
                let _ = send.send(self.reauth_session(id));
            }
            SessionCommand::Delete(id, send) => {
                let _ = send.send(self.delete_session(id));
            }
            SessionCommand::Guest(send) => {
                let _ = send.send(self.guest_session());
            }
        }
    }
}

impl SessionStore {
    fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            users: GroupMap::new(),
            guests: HashSet::new(),
        }
    }

    fn generate_session(&mut self) -> SessionToken {
        let mut digest = SessionToken::default();
        self.rng.fill_bytes(&mut digest.0);
        digest
    }

    fn create_session(&mut self, id: SquireAccountId) -> SessionToken {
        let token = self.generate_session();
        self.users.insert(token.clone(), id);
        token
    }

    fn guest_session(&mut self) -> SessionToken {
        let token = self.generate_session();
        self.guests.insert(token.clone());
        token
    }

    fn get_session(&mut self, token: SessionToken) -> SquireSession {
        if let Some(id) = self.users.get_right(&token) {
            SquireSession::Active(*id)
        } else if self.guests.contains(&token) {
            SquireSession::Guest(token)
        } else {
            SquireSession::UnknownUser
        }
    }

    fn reauth_session(&mut self, user: AnyUser) -> SessionToken {
        match user {
            AnyUser::Guest(token) => {
                self.guests.remove(&token);
                self.guest_session()
            }
            AnyUser::Active(token) => {
                if let Some(&id) = self.users.get_right(&token) {
                    self.users.remove_left(&token);
                    let token = self.generate_session();
                    self.users.insert(token.clone(), id);
                    token
                } else {
                    self.generate_session()
                }
            }
            AnyUser::Expired(token) => {
                // TODO: Replace users with expired sessions
                if let Some(&id) = self.users.get_right(&token) {
                    self.users.remove_left(&token);
                    let token = self.generate_session();
                    self.users.insert(token.clone(), id);
                    token
                } else {
                    self.generate_session()
                }
            }
        }
    }

    fn delete_session(&mut self, user: AnyUser) -> bool {
        match user {
            AnyUser::Guest(token) => self.guests.remove(&token),
            AnyUser::Active(token) => self.users.remove_left(&token).is_some(),
            AnyUser::Expired(token) => {
                // TODO: This needs to reference the recently expired sessions
                self.users.remove_left(&token).is_some()
            }
        }
    }
}

impl Trackable<SquireAccountId, SessionToken> for SessionCommand {
    fn track(msg: SquireAccountId, send: OneshotSender<SessionToken>) -> Self {
        Self::Create(msg, send)
    }
}

impl Trackable<(), SessionToken> for SessionCommand {
    fn track((): (), send: OneshotSender<SessionToken>) -> Self {
        Self::Guest(send)
    }
}

impl Trackable<SessionToken, SquireSession> for SessionCommand {
    fn track(msg: SessionToken, send: OneshotSender<SquireSession>) -> Self {
        Self::Get(msg, send)
    }
}

impl Trackable<AnyUser, SessionToken> for SessionCommand {
    fn track(msg: AnyUser, send: OneshotSender<SessionToken>) -> Self {
        Self::Reauth(msg, send)
    }
}

impl Trackable<AnyUser, bool> for SessionCommand {
    fn track(msg: AnyUser, send: OneshotSender<bool>) -> Self {
        Self::Delete(msg, send)
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
