mod init;
mod requests;
pub mod tournaments;
mod utils;

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc, cell::RefCell,
};

use async_session::{async_trait, MemoryStore, SessionStore};
use dashmap::DashMap;
use mtgjson::mtgjson::{atomics::Atomics, meta::Meta};
use squire_lib::{
    identifiers::SquireAccountId,
    operations::{OpSync, Rollback, RollbackError, SyncStatus},
    tournament::{TournamentId, TournamentPreset},
    tournament_manager::TournamentManager,
};

use crate::{
    accounts::VerificationData,
    version::{ServerMode, Version},
};

use super::{state::ServerState, User};

#[derive(Debug, Clone)]
pub struct AppState {
    store: MemoryStore,
    users: DashMap<SquireAccountId, User>,
    verified: DashMap<SquireAccountId, VerificationData>,
    tourns: DashMap<TournamentId, TournamentManager>,
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
        self.verified.get(&user.account.id).map(|data| (*data).clone())
    }

    async fn create_tournament(
        &self,
        user: User,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentManager {
        let tourn = TournamentManager::new(user.account, name, preset, format);
        self.tourns.insert(tourn.id, tourn.clone());
        tourn
    }

    async fn query_tournament<F, O>(&self, id: &TournamentId, f: F) -> Option<O>
    where
        F: Send + FnOnce(&TournamentManager) -> O,
    {
        self.tourns.get(id).map(|t| (f)(&*t))
    }

    async fn create_verification_data(&self, user: &User) -> String {
        let data = VerificationData {
            confirmation: "ABCDEF".to_owned(),
            status: true,
        };
        self.verified.insert(user.account.id, data.clone());
        data.confirmation
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
        self.tourns.get_mut(id).map(|mut t| t.propose_rollback(rollback))
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
