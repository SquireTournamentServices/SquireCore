use std::{error::Error, sync::Arc};

use async_session::{async_trait, SessionStore};
use squire_lib::tournament::TournamentSeed;

use crate::{
    model::{
        identifiers::SquireAccountId,
        tournament::{TournamentId, TournamentPreset},
    },
    server::User,
    sync::{OpSync, TournamentManager},
    version::Version,
};

#[async_trait]
pub trait ServerState: SessionStore + Clone + Send + Sync {
    fn get_version(&self) -> Version;

    async fn create_tourn(&self, user: User, seed: TournamentSeed) -> TournamentManager;

    async fn get_tourn(&self, id: TournamentId) -> Option<TournamentManager>;

    async fn persist_tourn(&self, tourn: &TournamentManager) -> bool;

    async fn bulk_persist<I, R>(&self, iter: I) -> bool
    where
        I: Send + Iterator<Item = R>,
        R: Send + Sync + AsRef<TournamentManager>,
    {
        let mut digest = true;
        for tourn in iter {
            digest &= self.persist_tourn(tourn.as_ref()).await;
            if !digest {
                break;
            }
        }
        digest
    }
}
