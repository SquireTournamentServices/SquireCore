use std::{borrow::Cow, ops::Range, sync::Arc};

use async_trait::async_trait;
use axum::extract::ws::WebSocket;
use mongodb::{options::ClientOptions, Client as DbClient, Database};
use squire_sdk::{
    actor::{ActorBuilder, ActorClient},
    api::*,
    model::{
        accounts::SquireAccount,
        identifiers::{SquireAccountId, TournamentId},
    },
    server::{
        gathering::{GatheringHall, GatheringHallMessage},
        session::{AnyUser, SessionWatcher, SquireSession},
        state::ServerState,
    },
    sync::TournamentManager,
};

mod accounts;
mod boilerplate;
mod session;
mod tournaments;
mod user_profile;

pub use accounts::*;
pub use session::*;
pub use tournaments::*;
// pub use user_profile::*;

pub type Uri = Cow<'static, str>;
pub type DbName = Option<String>;

/// A builder for an `AppState`.
#[derive(Debug, Clone)]
pub struct AppStateBuilder<T, N> {
    db_conn: T,
    db_name: N,
    tourn_coll: Option<String>,
}

impl AppStateBuilder<(), ()> {
    /// Constructs an `AppStateBuilder` that uses the default MongoDB URL.
    pub fn new() -> AppStateBuilder<Uri, DbName> {
        AppStateBuilder {
            db_conn: Cow::Borrowed("mongodb://localhost:27017"),
            db_name: None,
            tourn_coll: None,
        }
    }
}

impl AppStateBuilder<Uri, DbName> {
    /// Creates a builder that hold the URL of the MongoDB instance. A connection will be
    /// established upon building of the `AppState`
    #[allow(dead_code)]
    pub fn with_address<S: ToString>(uri: S) -> AppStateBuilder<Uri, DbName> {
        AppStateBuilder {
            db_conn: Cow::Owned(uri.to_string()),
            db_name: None,
            tourn_coll: None,
        }
    }

    /// Sets the address used as the MongoDB connection string. Default is
    /// `mongodb://localhost:27017`.
    #[allow(dead_code)]
    pub fn address<S: ToString>(mut self, addr: S) -> Self {
        self.db_conn = Cow::Owned(addr.to_string());
        self
    }

    /// Sets the name of the database. Default is `Squire`, or `SquireTesting` if the crate is
    /// compiled for testing.
    #[allow(dead_code)]
    pub fn database_name(mut self, name: impl Into<String>) -> Self {
        self.db_name = Some(name.into());
        self
    }

    #[cfg(not(test))]
    fn get_db_name(&self) -> &str {
        self.db_name.as_deref().unwrap_or("Squire")
    }

    #[cfg(test)]
    fn get_db_name(&self) -> &str {
        self.db_name.as_deref().unwrap_or("SquireTesting")
    }

    /// Constructs an `AppState` by trying to connect to the DB via the held address.
    ///
    /// # Panics
    /// Panics if a connection can not be established
    pub async fn build(self) -> AppState {
        let client_options = ClientOptions::parse(&self.db_conn).await.unwrap();
        let db_conn = DbClient::with_options(client_options)
            .unwrap()
            .database(self.get_db_name());
        let tourn_coll = Arc::from(self.get_tournament_collection_name());
        let tourn_db = TournDb::new(db_conn.clone(), tourn_coll);
        let tournaments = ActorClient::builder(TournPersister::new(tourn_db.clone())).launch();
        let gatherings = ActorBuilder::new(GatheringHall::new(tournaments.clone())).launch();
        AppState {
            sessions: SessionStoreHandle::new(db_conn.clone()),
            accounts: AccountStoreHandle::new(db_conn),
            gatherings,
            tourn_db,
        }
    }
}

impl AppStateBuilder<Database, ()> {
    /// Creates a builder that holds a DB client
    pub fn with_db(db: Database) -> AppStateBuilder<Database, ()> {
        AppStateBuilder {
            db_conn: db,
            db_name: (),
            tourn_coll: None,
        }
    }

    /// Constructs an `AppState` using the held DB client.
    pub fn build(self) -> AppState {
        let tourn_coll: Arc<str> = Arc::from(self.get_tournament_collection_name());
        let tourn_db = TournDb::new(self.db_conn.clone(), tourn_coll);
        let tourns = ActorClient::builder(TournPersister::new(tourn_db.clone())).launch();
        let gatherings = ActorBuilder::new(GatheringHall::new(tourns.clone())).launch();
        AppState {
            sessions: SessionStoreHandle::new(self.db_conn.clone()),
            accounts: AccountStoreHandle::new(self.db_conn),
            gatherings,
            tourn_db,
        }
    }
}

impl<T, S> AppStateBuilder<T, S> {
    /// Sets the name of the collection used for storing tournaments. Default is `Tournaments`.
    #[allow(dead_code)]
    pub fn tournament_collection_name(mut self, name: impl Into<String>) -> Self {
        self.tourn_coll = Some(name.into());
        self
    }

    fn get_tournament_collection_name(&self) -> &str {
        self.tourn_coll.as_deref().unwrap_or("Tournaments")
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    tourn_db: TournDb,
    sessions: SessionStoreHandle,
    accounts: AccountStoreHandle,
    gatherings: ActorClient<GatheringHall<TournPersister>>,
}

impl AppState {
    pub async fn new() -> Self {
        AppStateBuilder::new().build().await
    }

    pub fn get_db(&self) -> Database {
        self.tourn_db.get_db()
    }

    pub async fn login(&self, cred: Credentials) -> Result<SessionToken, LoginError> {
        match self.accounts.authenticate(cred).await {
            Some(id) => Ok(self.sessions.create(id).await),
            None => Err(LoginError),
        }
    }

    pub async fn create_account(&self, form: RegForm) -> SquireAccountId {
        self.accounts.create(form).await
    }

    pub async fn get_account(&self, id: SquireAccountId) -> Option<SquireAccount> {
        self.accounts.get(id).await
    }

    pub async fn get_account_by_session(&self, token: SessionToken) -> Option<SquireAccount> {
        if let SquireSession::Active(id) = self.sessions.get(token).await {
            self.get_account(id).await
        } else {
            None
        }
    }

    pub async fn delete_account(&self, id: SquireAccountId) -> bool {
        self.accounts.delete(id).await
    }
}

#[async_trait]
impl ServerState for AppState {
    fn get_version(&self) -> Version {
        Version {
            version: "v0.1.0".into(),
            mode: ServerMode::Extended,
        }
    }

    async fn get_tourn_summaries(&self, including: Range<usize>) -> Vec<TournamentSummary> {
        self.tourn_db.get_tourn_summaries(including).await
    }

    async fn get_tourn(&self, id: TournamentId) -> Option<TournamentManager> {
        self.tourn_db.get_tourn(id).await.map(|tourn| *tourn)
    }

    async fn persist_tourn(&self, tourn: &TournamentManager) -> bool {
        self.tourn_db.persist_tourn(tourn).await
    }

    async fn handle_new_onlooker(&self, id: TournamentId, user: SessionWatcher, ws: WebSocket) {
        println!("Passing connection request off to gathering hall...");
        self.gatherings
            .send(GatheringHallMessage::NewConnection(id, user, ws))
    }

    async fn get_session(&self, token: SessionToken) -> SquireSession {
        self.sessions.get(token).await
    }

    async fn create_session(&self, id: SquireAccountId) -> SessionToken {
        self.sessions.create(id).await
    }

    async fn guest_session(&self) -> SessionToken {
        self.sessions.guest().await
    }

    async fn reauth_session(&self, user: AnyUser) -> SessionToken {
        self.sessions.reauth(user).await
    }

    async fn terminate_session(&self, user: AnyUser) -> bool {
        self.sessions.delete(user).await
    }

    async fn watch_session(&self, user: AnyUser) -> Option<SessionWatcher> {
        self.sessions
            .watch(user.into_token())
            .await
            .map(SessionWatcher::new)
    }
}
