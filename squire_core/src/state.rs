use std::sync::Arc;

use async_session::{async_trait, MemoryStore, SessionStore};
use mongodb::{options::ClientOptions, Client as DbClient, Database, Collection};
use squire_sdk::{
    accounts::{SquireAccount, SquireAccountId, VerificationData},
    cards::{atomics::Atomics, meta::Meta},
    server::{state::ServerState, User},
    tournaments::{
        OpSync, Rollback, RollbackError, SyncStatus, TournamentId, TournamentManager,
        TournamentPreset,
    },
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
}

#[async_trait]
impl ServerState for AppState {
    fn get_version(&self) -> Version {
        Version {
            version: "0.1.0-pre-alpha".to_string(),
            mode: ServerMode::Extended,
        }
    }

    fn get_verification_data(&self, user: &User) -> Option<VerificationData> {
        None
    }

    async fn create_tournament(
        &self,
        user: User,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentManager {
        todo!()
    }

    async fn query_tournament<F, O>(&self, id: &TournamentId, f: F) -> Option<O>
    where
        F: Send + FnOnce(&TournamentManager) -> O,
    {
        todo!()
    }

    async fn create_verification_data(&self, user: &User) -> VerificationData {
        todo!()
    }

    async fn sync_tournament(
        &self,
        id: &TournamentId,
        user: &User,
        sync: OpSync,
    ) -> Option<SyncStatus> {
        todo!()
    }

    async fn rollback_tournament(
        &self,
        id: &TournamentId,
        user: &User,
        rollback: Rollback,
    ) -> Option<Result<(), RollbackError>> {
        todo!()
    }

    async fn load_user(&self, user: User) {
        todo!()
    }

    async fn get_user(&self, id: &SquireAccountId) -> Option<User> {
        todo!()
    }

    async fn get_cards_meta(&self) -> Meta {
        todo!()
    }

    async fn get_atomics(&self) -> Arc<Atomics> {
        todo!()
    }

    async fn update_cards(&self) -> Result<(), Box<dyn std::error::Error>> {
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
