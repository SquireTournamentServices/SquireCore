use std::{error::Error, sync::Arc};

use async_session::{async_trait, SessionStore};
use squire_lib::{
    operations::{OpSync, Rollback, RollbackError, SyncStatus},
    tournament::TournamentId,
    tournament_manager::TournamentManager,
};

use crate::{
    accounts::VerificationData,
    cards::{atomics::Atomics, meta::Meta},
    model::{identifiers::SquireAccountId, tournament::TournamentPreset},
    server::{AppState, User},
    tournaments::SyncRequest,
    version::Version,
};

#[async_trait]
pub trait ServerState: SessionStore {
    fn get_version(&self) -> Version;
    fn get_verification_data(&self, user: &User) -> Option<VerificationData>;
    async fn create_tournament(
        &self,
        user: User,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentManager;
    async fn query_tournament<F, O>(&self, id: &TournamentId, f: F) -> Option<O>
    where
        F: Send + FnOnce(&TournamentManager) -> O;
    async fn create_verification_data(&self, user: &User) -> String;
    async fn sync_tournament(
        &self,
        id: &TournamentId,
        user: &User,
        sync: OpSync,
    ) -> Option<SyncStatus>;
    async fn rollback_tournament(
        &self,
        id: &TournamentId,
        user: &User,
        rollback: Rollback,
    ) -> Option<Result<(), RollbackError>>;
    async fn get_user(&self, id: &SquireAccountId) -> Option<User>;
    async fn get_cards_meta(&self) -> Meta;
    async fn get_atomics(&self) -> Arc<Atomics>;
    async fn update_cards(&self) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl ServerState for AppState {
    fn get_version(&self) -> Version {
        todo!()
    }

    fn get_verification_data(&self, user: &User) -> Option<VerificationData> {
        todo!()
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

    async fn get_user(&self, id: &SquireAccountId) -> Option<User> {
        todo!()
    }

    async fn create_verification_data(&self, user: &User) -> String {
        todo!()
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
