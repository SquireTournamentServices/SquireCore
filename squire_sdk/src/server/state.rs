use std::ops::Range;

use async_trait::async_trait;
use http::HeaderValue;

use super::session::SquireSession;
use crate::{
    api::{TournamentSummary, Version},
    model::tournament::TournamentId,
    sync::TournamentManager,
};

#[async_trait]
pub trait ServerState: 'static + Clone + Send + Sync {
    fn get_version(&self) -> Version;

    async fn get_tourn_summaries(&self, including: Range<usize>) -> Vec<TournamentSummary>;

    async fn get_tourn(&self, id: TournamentId) -> Option<TournamentManager>;

    async fn persist_tourn(&self, tourn: &TournamentManager) -> bool;

    async fn get_session(&self, header: HeaderValue) -> SquireSession;

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
}
