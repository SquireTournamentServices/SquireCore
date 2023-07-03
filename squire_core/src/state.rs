use std::{borrow::Cow, sync::Arc};

use async_session::{async_trait, MemoryStore, SessionStore};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, spec::BinarySubtype, Binary, Document},
    options::{ClientOptions, ReplaceOptions, UpdateModifications, UpdateOptions},
    Client as DbClient, Collection, Database, IndexModel,
};
use squire_sdk::{
    model::{accounts::SquireAccount, identifiers::TypeId, tournament::TournamentSeed},
    server::{state::ServerState, User},
    tournaments::{OpSync, TournamentId, TournamentManager, TournamentPreset},
    version::{ServerMode, Version},
};
use tracing::Level;

/// Specifies how the Squire app connects to a MongoDB instance
#[derive(Debug, Clone, Default)]
pub struct AppSettings {
    address: Option<String>,
    database_name: Option<String>,
    tournament_collection_name: Option<String>,
}

impl AppSettings {
    /// Sets the address used as the MongoDB connection string. Default is
    /// `mongodb://localhost:27017`.
    pub fn address(mut self, addr: impl Into<Option<String>>) -> Self {
        self.address = addr.into();
        self
    }
    /// Sets the name of the database. Default is `Squire`, or `SquireTesting` if the crate is
    /// compiled for testing.
    pub fn database_name(mut self, name: impl Into<Option<String>>) -> Self {
        self.database_name = name.into();
        self
    }
    /// Sets the name of the collection used for storing tournaments. Default is `Tournaments`.
    pub fn tournament_collection_name(mut self, name: impl Into<Option<String>>) -> Self {
        self.tournament_collection_name = name.into();
        self
    }

    fn get_address(&self) -> &str {
        self.address
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("mongodb://localhost:27017")
    }
    #[cfg(not(test))]
    fn get_database_name(&self) -> &str {
        self.database_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Squire")
    }
    #[cfg(test)]
    fn get_database_name(&self) -> &str {
        self.database_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("SquireTesting")
    }
    fn get_tournament_collection_name(&self) -> &str {
        self.tournament_collection_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Tournaments")
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    client: DbClient,
    settings: AppSettings,
}

impl AppState {
    pub async fn new_with_settings(settings: AppSettings) -> Self {
        let mut client_options = ClientOptions::parse(settings.get_address()).await.unwrap();

        client_options.app_name = Some("SquireCore Public Server".to_string());

        let client = DbClient::with_options(client_options).unwrap();

        let slf = Self { client, settings };

        let index = IndexModel::builder().keys(doc! {"tourn.id": 1}).build();
        slf.get_tourns().create_index(index, None).await;

        slf
    }

    pub async fn new() -> Self {
        Self::new_with_settings(AppSettings::default()).await
    }

    pub fn get_db(&self) -> Database {
        self.client.database(self.settings.get_database_name())
    }

    pub fn get_tourns(&self) -> Collection<TournamentManager> {
        self.get_db()
            .collection(self.settings.get_tournament_collection_name())
    }

    fn make_query(id: TournamentId) -> Document {
        let b = Binary {
            bytes: id.as_bytes().to_vec(),
            subtype: BinarySubtype::Generic,
        };
        let out = doc! {
            "tourn.id": b,
        };
        // let out = doc! {"tourn.id": {
        //     "$gte": Binary { bytes: id.as_bytes().to_vec(), subtype: BinarySubtype::Generic },
        //     "$lte": Binary { bytes: id.as_bytes().to_vec(), subtype: BinarySubtype::Reserved(127)},
        // }};

        // let out = doc! {"tourn.id": id.to_string()};
        println!("{out}");
        out
    }

    /*
    pub fn get_past_tourns(&self) -> Collection<CompressedTournament> {
        self.get_db().collection("PastTournaments")
    }

    pub async fn query_all_past_tournaments<F, O, Out>(&self, mut f: F) -> Out
    where
        Out: FromIterator<O>,
        O: Send,
        F: Send + FnMut(&CompressedTournament) -> O,
    {
        let mut digest = Vec::new();
        let mut cursor = self.get_past_tourns().find(None, None).await.unwrap();
        while let Some(tourn) = cursor.try_next().await.unwrap() {
            digest.push(f(&tourn));
        }
        digest.into_iter().collect()
    }
    */
}

#[async_trait]
impl ServerState for AppState {
    fn get_version(&self) -> Version {
        Version {
            version: "0.1.0-pre-alpha".to_string(),
            mode: ServerMode::Extended,
        }
    }

    async fn create_tourn(&self, user: User, seed: TournamentSeed) -> TournamentManager {
        todo!()
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
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
        {
            Ok(result) => result.matched_count != 0,
            Err(e) => {
                tracing::event!(Level::WARN, "Could not persist tournament: {e}");
                false
            }
        }
    }
}

#[async_trait]
impl SessionStore for AppState {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> async_session::Result<Option<async_session::Session>> {
        todo!()
    }

    async fn store_session(
        &self,
        session: async_session::Session,
    ) -> async_session::Result<Option<String>> {
        todo!()
    }

    async fn destroy_session(&self, session: async_session::Session) -> async_session::Result {
        todo!()
    }

    async fn clear_store(&self) -> async_session::Result {
        todo!()
    }
}
