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
    collections::hash_map::HashMap,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use futures::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{UpdateModifications, UpdateOptions},
    Collection, Database,
};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use squire_sdk::{
    actor::*,
    api::SessionToken,
    model::identifiers::SquireAccountId,
    server::session::{AnyUser, SquireSession},
};
use tokio::sync::{
    oneshot::Sender as OneshotSender,
    watch::{channel, Receiver as Watcher, Sender as Broadcaster},
};
use derive_more::From;
use tracing::Level;

#[derive(From)]
pub enum SessionCommand {
    Create(SquireAccountId, OneshotSender<SessionToken>),
    Guest(OneshotSender<SessionToken>),
    Get(SessionToken, OneshotSender<SquireSession>),
    Reauth(AnyUser, OneshotSender<SessionToken>),
    Delete(AnyUser, OneshotSender<bool>),
    Subscribe(SessionToken, OneshotSender<Option<Watcher<SquireSession>>>),
    #[from(ignore)]
    Expiry(SessionToken),
    #[from(ignore)]
    Revoke(SessionToken),
}

impl From<((), OneshotSender<SessionToken>)> for SessionCommand {
    fn from(((), send): ((), OneshotSender<SessionToken>)) -> Self {
        Self::Guest(send)
    }
}

pub struct SessionStore {
    rng: StdRng,
    db: SessionDb,
    comms: HashMap<SessionToken, Broadcaster<SquireSession>>,
    sessions: HashMap<SessionToken, Session>,
}

#[derive(Debug, Clone)]
pub struct SessionDb {
    db: Database,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
struct Session {
    /// The time that the session was created
    epoch: DateTime<Utc>,
    /// The bytes that make up the identifier of the token
    token: SessionToken,
    /// If the session belongs to user, this is their account id.
    id: Option<SquireAccountId>,
}

#[async_trait]
impl ActorState for SessionStore {
    type Message = SessionCommand;

    async fn start_up(&mut self, scheduler: &mut Scheduler<Self>) {
        self.db.clone().load_all_sessions(self).await;
        // Schedule the expiry and revocation times in the scheduler
        self.sessions.values().for_each(|s| {
            scheduler.schedule(s.next_deadline(), SessionCommand::Expiry(s.token.clone()))
        });
        // Session watch channels are created lazily
    }

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        println!("Got session message: {msg:?}");
        match msg {
            SessionCommand::Create(id, send) => {
                drop(send.send(self.create_session(scheduler, id).token))
            }
            SessionCommand::Get(token, send) => drop(send.send(self.get_session(token))),
            SessionCommand::Reauth(id, send) => drop(send.send(self.reauth_session(scheduler, id))),
            SessionCommand::Delete(id, send) => drop(send.send(self.delete_session(scheduler, id))),
            SessionCommand::Guest(send) => drop(send.send(self.guest_session(scheduler).token)),
            SessionCommand::Subscribe(token, send) => drop(send.send(self.sub_to_session(&token))),
            SessionCommand::Expiry(token) => self.expire_session(scheduler, token),
            SessionCommand::Revoke(token) => self.revoke_session(scheduler, &token),
        }
    }
}

impl SessionStore {
    pub fn new(db: Database) -> Self {
        let db = SessionDb::new(db);
        Self {
            db,
            rng: StdRng::from_entropy(),
            comms: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    fn generate_session(&mut self, scheduler: &mut Scheduler<Self>) -> SessionToken {
        let mut digest = SessionToken::default();
        self.rng.fill_bytes(&mut digest.0);
        let deadline = Instant::now() + Session::SESSION_DUR;
        scheduler.schedule(deadline, SessionCommand::Expiry(digest.clone()));
        digest
    }

    fn create_session(&mut self, scheduler: &mut Scheduler<Self>, id: SquireAccountId) -> Session {
        let token = self.generate_session(scheduler);
        let session = Session::new_with_id(token.clone(), id);
        self.sessions.insert(token.clone(), session.clone());
        let db = self.db.clone();
        let db_session = session.clone();
        scheduler.process(async move { db.persist_session(db_session).await });
        session
    }

    fn guest_session(&mut self, scheduler: &mut Scheduler<Self>) -> Session {
        let token = self.generate_session(scheduler);
        let session = Session::new(token.clone());
        self.sessions.insert(token.clone(), session.clone());
        let db = self.db.clone();
        let db_session = session.clone();
        scheduler.process(async move { db.persist_session(db_session).await });
        session
    }

    fn get_session(&mut self, token: SessionToken) -> SquireSession {
        self.sessions
            .get(&token)
            .map(Session::as_squire_session)
            .unwrap_or_default()
    }

    fn reauth_session(&mut self, scheduler: &mut Scheduler<Self>, user: AnyUser) -> SessionToken {
        match user {
            AnyUser::Guest(token) => {
                self.sessions.remove(&token);
                let session = self.guest_session(scheduler);
                if let Some(sq_sess) = self.comms.get(&token) {
                    sq_sess.send_replace(session.as_squire_session());
                }
                session.token
            }
            AnyUser::Active(token) | AnyUser::Expired(token) | AnyUser::ExpiredGuest(token) => {
                match self.sessions.remove(&token).and_then(|s| s.id) {
                    Some(id) => {
                        let session = self.create_session(scheduler, id);
                        if let Some(sq_sess) = self.comms.get(&token) {
                            sq_sess.send_replace(session.as_squire_session());
                        }
                        session.token
                    }
                    None => self.generate_session(scheduler),
                }
            }
        }
    }

    fn delete_session(&mut self, scheduler: &mut Scheduler<Self>, user: AnyUser) -> bool {
        match user {
            AnyUser::Guest(token) | AnyUser::Active(token) => {
                if let Some(session) = self.remove_session(&token) {
                    let db = self.db.clone();
                    scheduler.process(async move { db.remove_session(session).await });
                    true
                } else {
                    false
                }
            }
            AnyUser::ExpiredGuest(token) | AnyUser::Expired(token) => {
                if let Some(session) = self.remove_session(&token) {
                    let db = self.db.clone();
                    scheduler.process(async move { db.remove_expired_session(session).await });
                    true
                } else {
                    false
                }
            }
        }
    }

    fn expire_session(&mut self, scheduler: &mut Scheduler<Self>, token: SessionToken) {
        if let Some(session) = self.sessions.get(&token).cloned() {
            // Update listeners to the session
            if let Some(sq_sess) = self.comms.get_mut(&token) {
                sq_sess.send_replace(session.as_squire_session());
            }
            scheduler.schedule(
                session.next_deadline(),
                SessionCommand::Revoke(token.clone()),
            );
            let db = self.db.clone();
            scheduler.process(async move { db.expire_session(session).await });
        }
    }

    fn revoke_session(&mut self, scheduler: &mut Scheduler<Self>, token: &SessionToken) {
        if let Some(session) = self.remove_session(token) {
            let db = self.db.clone();
            scheduler.process(async move { db.remove_expired_session(session).await });
        }
    }

    fn remove_session(&mut self, token: &SessionToken) -> Option<Session> {
        if let Some(sq_sess) = self.comms.get_mut(token) {
            sq_sess.send_replace(SquireSession::NotLoggedIn);
        }
        self.comms.remove(token);
        self.sessions.remove(token)
    }

    fn sub_to_session(&mut self, token: &SessionToken) -> Option<Watcher<SquireSession>> {
        self.comms
            .get(token)
            .map(|comm| comm.subscribe())
            .or_else(|| self.create_watcher(token))
    }

    fn create_watcher(&mut self, token: &SessionToken) -> Option<Watcher<SquireSession>> {
        let session = self.sessions.get(token)?;
        let sq_sess = session.as_squire_session();
        let (send, recv) = channel(sq_sess);
        self.comms.insert(token.clone(), send);
        Some(recv)
    }
}

impl SessionDb {
    const ACTIVE_SESSION_TABLE: &'static str = "ActiveSessions";
    const EXPIRED_SESSION_TABLE: &'static str = "ExpiredSessions";

    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_active_table(&self) -> Collection<Session> {
        self.db.collection(Self::ACTIVE_SESSION_TABLE)
    }

    fn get_expired_table(&self) -> Collection<Session> {
        self.db.collection(Self::EXPIRED_SESSION_TABLE)
    }

    /// Takes mutable reference to a session store and fills its cache with all the sessions in the
    /// database. This is used on startup.
    pub async fn load_all_sessions(&self, cache: &mut SessionStore) {
        // Fetch all active sessions
        let mut cursor = self.get_active_table().find(None, None).await.unwrap();
        while let Some(session) = cursor.next().await {
            if let Ok(session) = session {
                let token = session.token.clone();
                cache.sessions.insert(token, session);
            }
        }
        // Fetch all expired sessions
        let mut cursor = self.get_expired_table().find(None, None).await.unwrap();
        while let Some(session) = cursor.next().await {
            if let Ok(session) = session {
                let token = session.token.clone();
                cache.sessions.insert(token, session);
            }
        }
    }

    /// Inserts or updates a session in the database.
    async fn persist_session(&self, session: Session) {
        let table = self.get_active_table();
        persist_session(table, session).await;
    }

    /// Updates a session in the database by marking it as expired.
    async fn expire_session(&self, session: Session) {
        let table = self.get_active_table();
        if delete_session(table, session.clone()).await {
            let table = self.get_expired_table();
            persist_session(table, session).await;
        }
    }

    /// Removes an active session from the database.
    async fn remove_session(&self, session: Session) {
        let table = self.get_active_table();
        delete_session(table, session).await;
    }

    /// Removes an expired session from the database.
    async fn remove_expired_session(&self, session: Session) {
        let table = self.get_expired_table();
        delete_session(table, session).await;
    }
}

async fn persist_session(table: Collection<Session>, session: Session) -> bool {
    let doc: Document = mongodb::bson::to_raw_document_buf(&session)
        .unwrap()
        .try_into()
        .unwrap();
    if table
        .update_one(
            doc.clone(),
            UpdateModifications::Document(doc! {"$set": doc}),
            UpdateOptions::builder().upsert(true).build(),
        )
        .await
        .is_err()
    {
        if let Err(err) = table.insert_one(session.clone(), None).await {
            tracing::event!(
                Level::WARN,
                "Could not persist session `{session:?}` got error: {err}",
            );
            return false;
        }
    }
    true
}

async fn delete_session(table: Collection<Session>, session: Session) -> bool {
    let doc: Document = mongodb::bson::to_raw_document_buf(&session)
        .unwrap()
        .try_into()
        .unwrap();
    table.delete_one(doc, None).await.is_ok()
}

#[derive(Debug, Clone)]
pub struct SessionStoreHandle {
    client: ActorClient<SessionStore>,
}

impl SessionStoreHandle {
    pub fn new(db: Database) -> Self {
        let client = ActorClient::builder(SessionStore::new(db)).launch();
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

    pub fn watch(&self, token: SessionToken) -> Tracker<Option<Watcher<SquireSession>>> {
        self.client.track(token)
    }
}

impl Session {
    /// The amount of time a session can live for before being marked as expired (6 days)
    const SESSION_DUR: Duration = Duration::from_secs(518400);
    /// The amount of time an expired session can live for before being forgotten entirely (1 day)
    const EXPIRY_DUR: Duration = Duration::from_secs(86400);

    fn new(token: SessionToken) -> Self {
        Self {
            epoch: Utc::now(),
            token,
            id: None,
        }
    }

    fn new_with_id(token: SessionToken, id: SquireAccountId) -> Self {
        Self {
            epoch: Utc::now(),
            token,
            id: Some(id),
        }
    }

    /// Returns the next deadline for this session (expiry time for active sessions and revocation
    /// time for expired sessions). If a session should already be removed, this returns
    /// `Instant::now()`.
    fn next_deadline(&self) -> Instant {
        // The amount of time that has passed since the creation of the session.
        let elapsed = self.get_elapsed_dur();
        Instant::now()
            + Self::SESSION_DUR
                .checked_sub(elapsed)
                .or_else(|| (Self::SESSION_DUR + Self::EXPIRY_DUR).checked_sub(elapsed))
                .unwrap_or_default()
    }

    fn is_active(&self) -> bool {
        Self::SESSION_DUR > self.get_elapsed_dur()
    }

    fn get_elapsed_dur(&self) -> Duration {
        (Utc::now() - self.epoch).to_std().unwrap_or_default()
    }

    /// Creates a SquireSession
    fn as_squire_session(&self) -> SquireSession {
        match self.id {
            Some(id) if self.is_active() => SquireSession::Active(id),
            Some(id) => SquireSession::Expired(id),
            None if self.is_active() => SquireSession::Guest(self.token.clone()),
            None => SquireSession::ExpiredGuest(self.token.clone()),
        }
    }
}
