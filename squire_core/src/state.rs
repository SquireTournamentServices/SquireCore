use std::sync::Arc;

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

#[derive(Debug, Clone)]
pub struct AppState {
    client: DbClient,
}

impl AppState {
    pub async fn new() -> Self {
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .unwrap();

        client_options.app_name = Some("SquireCore Public Server".to_string());

        let client = DbClient::with_options(client_options).unwrap();

        let slf = Self { client };

        let index = IndexModel::builder().keys(doc! {"tourn.id": 1}).build();
        slf.get_tourns().create_index(index, None).await;

        slf
    }

    #[cfg(not(test))]
    pub fn get_db(&self) -> Database {
        self.client.database("Squire")
    }

    #[cfg(test)]
    pub fn get_db(&self) -> Database {
        self.client.database("SquireTesting")
    }

    pub fn get_tourns(&self) -> Collection<TournamentManager> {
        self.get_db().collection("Tournaments")
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
        let doc: Document = bson::to_raw_document_buf(tourn)
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

#[cfg(test)]
mod tests {
    use async_session::chrono::Utc;
    use squire_sdk::{
        server::state::ServerState,
        tournaments::{TournOp, TournamentManager},
    };

    use super::AppState;

    async fn clear_database() {
        AppState::new().await.get_db().drop(None).await;
    }

    #[tokio::test]
    async fn insert_fetch_tourn() {
        clear_database().await;

        let manager =
            TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());
        let state = AppState::new().await;

        state.persist_tourn(&manager).await;
        let retrieved_tourn = state
            .get_tourn(manager.id)
            .await
            .expect("Could not retrieve tournament from database");

        assert_eq!(manager, retrieved_tourn);
    }

    #[tokio::test]
    async fn check_already_persisted() {
        clear_database().await;

        let mut manager =
            TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());
        let state = AppState::new().await;

        assert!(!state.persist_tourn(&manager).await);
        assert!(state.persist_tourn(&manager).await);
    }
}
