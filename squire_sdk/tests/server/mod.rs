#![allow(unused)]

pub mod init;
pub mod requests;
pub mod tournaments;
pub mod utils;

use std::{error::Error, sync::Arc};

use async_session::{async_trait, MemoryStore, SessionStore};
use dashmap::DashMap;
use mtgjson::mtgjson::{atomics::Atomics, meta::Meta};
use squire_sdk::{
    api::*,
    model::{
        identifiers::SquireAccountId,
        tournament::{TournamentId, TournamentPreset},
    },
    server::{state::ServerState, User},
    sync::{OpSync, TournamentManager},
};

/*
// `MemoryStore` is ephemeral and will not persist between test runs
#[derive(Debug, Clone)]
pub struct AppState {
    pub store: MemoryStore,
    pub users: Arc<DashMap<SquireAccountId, User>>,
    pub verified: Arc<DashMap<SquireAccountId, VerificationData>>,
    pub tourns: Arc<DashMap<TournamentId, TournamentManager>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            store: MemoryStore::new(),
            users: Arc::new(DashMap::new()),
            verified: Arc::new(DashMap::new()),
            tourns: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl ServerState for AppState {
    fn get_version(&self) -> Version {
        Version {
            version: "0.1.0-pre-alpha".to_owned(),
            mode: ServerMode::Basic,
        }
    }

    fn get_verification_data(&self, user: &User) -> Option<VerificationData> {
        self.verified
            .get(&user.account.id)
            .map(|data| (*data).clone())
    }

    async fn create_tournament(
        &self,
        user: User,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentManager {
        let tourn = TournamentManager::new(user.account, get_seed());
        self.tourns.insert(tourn.id, tourn.clone());
        tourn
    }

    async fn query_tournament<F, O>(&self, id: &TournamentId, f: F) -> Option<O>
    where
        F: Send + FnOnce(&TournamentManager) -> O,
    {
        self.tourns.get(id).map(|t| (f)(&t))
    }

    async fn query_all_tournaments<F, O, Out>(&self, mut f: F) -> Out
    where
        Out: FromIterator<O>,
        F: Send + FnMut(&TournamentManager) -> O,
    {
        self.tourns.iter().map(|t| (f)(&t)).collect()
    }

    async fn create_verification_data(&self, user: &User) -> VerificationData {
        let data = VerificationData {
            confirmation: "ABCDEF".to_owned(),
            status: true,
        };
        self.verified.insert(user.account.id, data.clone());
        data
    }

    async fn sync_tournament(
        &self,
        id: &TournamentId,
        user: &User,
        sync: OpSync,
    ) -> Option<SyncStatus> {
        self.tourns.get_mut(id).map(|mut t| t.attempt_sync(sync))
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
        self.users.entry(user.account.id).or_insert(user);
    }

    async fn get_user(&self, id: &SquireAccountId) -> Option<User> {
        self.users.get(id).map(|user| (*user).clone())
    }

    async fn get_cards_meta(&self) -> Meta {
        todo!()
    }

    async fn get_atomics(&self) -> Arc<Atomics> {
        todo!()
    }

    async fn update_cards(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

#[async_trait]
impl SessionStore for AppState {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> async_session::Result<Option<async_session::Session>> {
        self.store.load_session(cookie_value).await
    }

    async fn store_session(
        &self,
        session: async_session::Session,
    ) -> async_session::Result<Option<String>> {
        self.store.store_session(session).await
    }

    async fn destroy_session(&self, session: async_session::Session) -> async_session::Result {
        self.store.destroy_session(session).await
    }

    async fn clear_store(&self) -> async_session::Result {
        self.store.clear_store().await
    }
}
*/
