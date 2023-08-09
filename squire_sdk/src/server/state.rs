use std::ops::Range;

use async_trait::async_trait;

use super::session::{AnySession, SquireSession, SessionToken};
use crate::{
    api::{Credentials, TournamentSummary, Version},
    model::tournament::TournamentId,
    sync::TournamentManager,
};

#[async_trait]
pub trait ServerState: 'static + Clone + Send + Sync {
    fn get_version(&self) -> Version;

    /* ------ Tournament-related methods ------ */
    async fn get_tourn_summaries(&self, including: Range<usize>) -> Vec<TournamentSummary>;

    async fn get_tourn(&self, id: TournamentId) -> Option<TournamentManager>;

    async fn persist_tourn(&self, tourn: &TournamentManager) -> bool;

    async fn bulk_persist<I>(&self, iter: I) -> bool
    where
        I: Send + Iterator<Item = TournamentManager>,
    {
        let mut digest = true;
        for tourn in iter {
            digest &= self.persist_tourn(&tourn).await;
            if !digest {
                break;
            }
        }
        digest
    }

    /* ------ Session-related methods ------ */
    async fn create_session(&self, cred: Credentials) -> SessionToken;

    async fn guest_session(&self) -> SessionToken;

    async fn get_session(&self, token: SessionToken) -> SquireSession;

    async fn reauth_session(&self, session: AnySession) -> SessionToken;

    async fn terminate_session(&self, session: AnySession) -> bool;
}
