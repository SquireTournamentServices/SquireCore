use std::{error::Error, sync::Arc};

use async_session::{async_trait, SessionStore};

use crate::{
    model::{
        identifiers::SquireAccountId,
        tournament::{TournamentId, TournamentPreset},
    },
    server::User,
    sync::{OpSync, Rollback, RollbackError, SyncStatus, TournamentManager},
    version::Version,
};

#[async_trait]
pub trait ServerState: SessionStore + Clone + Send + Sync {
    fn get_version(&self) -> Version;
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
    async fn query_all_tournaments<F, O, Out>(&self, f: F) -> Out
    where
        Out: FromIterator<O>,
        O: Send,
        F: Send + FnMut(&TournamentManager) -> O;
}
