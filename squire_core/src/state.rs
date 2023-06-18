use std::sync::Arc;

use async_session::{async_trait, MemoryStore, SessionStore};
use futures::stream::TryStreamExt;
use mongodb::{options::ClientOptions, Client as DbClient, Collection, Database};
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

        Self { client }
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
        todo!()
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
