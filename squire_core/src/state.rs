use std::sync::Arc;

use async_session::{async_trait, MemoryStore, SessionStore};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::doc, options::ClientOptions, Client as DbClient, Collection, Database, IndexModel,
};
use squire_sdk::{
    model::{accounts::SquireAccount, tournament::TournamentSeed},
    server::{state::ServerState, User},
    tournaments::{OpSync, TournamentId, TournamentManager, TournamentPreset},
    version::{ServerMode, Version},
};

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

        let index = IndexModel::builder().keys(doc! {"tourn_id": 1}).build();
        slf.get_tourns().create_index(index, None).await;

        slf
    }

    pub fn get_db(&self) -> Database {
        self.client.database("Squire")
    }

    pub fn get_tourns(&self) -> Collection<TournamentManager> {
        self.get_db().collection("Tournaments")
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
        let mut cursor = self.get_tourns().find(None, None).await.unwrap();
        while let Some(tourn) = cursor.try_next().await.unwrap() {
            if tourn.id == id {
                return Some(tourn);
            }
        }
        None
    }

    async fn persist_tourn(&self, tourn: &TournamentManager) -> bool {
        self.get_tourns().insert_one(tourn, None).await;
        false // TODO: wrong, but fix later
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
    use squire_sdk::tournaments::TournamentManager;

    use super::AppState;

    #[tokio::test]
    async fn insert_remove_tourn() {
        use squire_sdk::server::state::ServerState;

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
}
