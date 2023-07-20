use std::{borrow::Cow, ops::Range, sync::Arc};

use async_session::{async_trait, SessionStore};
use futures::StreamExt;
use mongodb::{
    bson::{doc, spec::BinarySubtype, Binary, Document},
    options::{ClientOptions, FindOptions, Hint, UpdateModifications, UpdateOptions},
    Client as DbClient, Collection, Database,
};
use squire_sdk::{
    model::{identifiers::TournamentId, tournament::TournamentSeed},
    server::{state::ServerState, User},
    sync::TournamentManager,
    tournaments::TournamentSummary,
    version::{ServerMode, Version},
};
use tracing::Level;

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
        let tourn_coll = Arc::from(self.get_tournament_collection_name());
        let client_options = ClientOptions::parse(&self.db_conn).await.unwrap();
        let db_conn = DbClient::with_options(client_options)
            .unwrap()
            .database(self.get_db_name());
        AppState {
            db_conn,
            tourn_coll,
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
        let tourn_coll = Arc::from(self.get_tournament_collection_name());
        AppState {
            db_conn: self.db_conn,
            tourn_coll,
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
    db_conn: Database,
    tourn_coll: Arc<str>,
}

impl AppState {
    const TOURN_INDEX_NAME: &str = "tourn_id";

    pub async fn new() -> Self {
        AppStateBuilder::new().build().await
    }

    pub fn get_db(&self) -> Database {
        self.db_conn.clone()
    }

    pub fn get_tourns(&self) -> Collection<TournamentManager> {
        self.get_db().collection(&self.tourn_coll)
    }

    fn make_query(id: TournamentId) -> Document {
        doc! { "tourn.id": Binary {
            bytes: id.as_bytes().to_vec(),
            subtype: BinarySubtype::Generic,
        }}
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

    async fn create_tourn(&self, _user: User, _seed: TournamentSeed) -> TournamentManager {
        todo!()
    }

    async fn get_tourn_summaries(&self, including: Range<usize>) -> Vec<TournamentSummary> {
        let Ok(cursor) = self
            .get_tourns()
            .find(
                None,
                FindOptions::builder().sort(doc! {"$natural":-1}).build(),
            )
            .await
        else {
            return vec![];
        };

        cursor
            .skip(including.start)
            .take(including.count())
            .filter_map(|u| async { u.ok().as_ref().map(TournamentSummary::from) })
            .collect()
            .await
    }

    async fn get_tourn(&self, id: TournamentId) -> Option<TournamentManager> {
        self.get_tourns()
            .find_one(Some(Self::make_query(id)), None)
            .await
            .ok()
            .flatten()
    }

    async fn persist_tourn(&self, tourn: &TournamentManager) -> bool {
        // There appears to be a problem in bson right now where `Collection::replace_one` uses the
        // normal document serializer, but `Collection::find_one` (and `Collection::insert_one` as
        // well) use the raw document serializer, which unfortunately behave differently. Therefore
        // `Collection::update_one` is used as a workaround so that we can call the raw document
        // serializer here
        let doc: Document = mongodb::bson::to_raw_document_buf(tourn)
            .unwrap()
            .try_into()
            .unwrap();
        match self
            .get_tourns()
            .update_one(
                Self::make_query(tourn.id),
                UpdateModifications::Document(doc! {"$set": doc}),
                UpdateOptions::builder()
                    .upsert(true)
                    .hint(Hint::Name(Self::TOURN_INDEX_NAME.to_string()))
                    .build(),
            )
            .await
        {
            Ok(result) => result.matched_count != 0,
            Err(e) => {
                tracing::event!(
                    Level::WARN,
                    r#"Could not persist tournament with name "{}" and id "{}" due to error: {e}"#,
                    tourn.tourn().name,
                    tourn.tourn().id,
                );
                false
            }
        }
    }
}

#[async_trait]
impl SessionStore for AppState {
    async fn load_session(
        &self,
        _cookie_value: String,
    ) -> async_session::Result<Option<async_session::Session>> {
        todo!()
    }

    async fn store_session(
        &self,
        _session: async_session::Session,
    ) -> async_session::Result<Option<String>> {
        todo!()
    }

    async fn destroy_session(&self, _session: async_session::Session) -> async_session::Result {
        todo!()
    }

    async fn clear_store(&self) -> async_session::Result {
        todo!()
    }
}
