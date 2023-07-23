use std::fmt::{self, Debug};

use reqwest::{header::CONTENT_TYPE, Client, Response};
use serde::Serialize;
use tokio::sync::broadcast::Receiver as Subscriber;

use self::{
    builder::ClientBuilder,
    import::ImportTracker,
    management_task::ManagementTaskSender,
    query::QueryTracker,
    update::{UpdateTracker, UpdateType},
};
use crate::{
    api::{LIST_TOURNAMENTS_ROUTE, TOURNAMENTS_ROUTE},
    model::{
        accounts::SquireAccount, identifiers::TournamentId, operations::TournOp,
        players::PlayerRegistry, rounds::RoundRegistry, tournament::TournamentSeed,
    },
    sync::TournamentManager,
    tournaments::TournamentSummary,
    version::ServerMode,
};

#[cfg(not(debug_assertions))]
/// The address of the service one Shuttle
pub const HOST_ADDRESS: &str = "s://squire.shuttleapp.rs";
#[cfg(debug_assertions)]
/// The address of the local hosh
pub const HOST_ADDRESS: &str = "://localhost:8000";

pub trait OnUpdate: 'static + Send + FnMut(TournamentId) {}

impl<T> OnUpdate for T where T: 'static + Send + FnMut(TournamentId) {}

pub mod builder;
pub mod compat;
pub mod error;
pub mod import;
pub mod management_task;
pub mod query;
pub mod subscription;
pub mod update;

pub struct SquireClient {
    client: Client,
    url: String,
    user: SquireAccount,
    server_mode: ServerMode,
    sender: ManagementTaskSender,
}

pub enum BackendImportStatus {
    /// The tournament was successfully sent to the backend and stored in the database.
    Success,
    /// The tournament was sent to the backend, but the backend around had a copy of it.
    AlreadyImported,
    /// The tournament was not found locally, so it could not be persisted.
    NotFound,
}

impl SquireClient {
    /// Returns a builder for the client
    pub fn builder() -> ClientBuilder<Box<dyn OnUpdate>, (), ()> {
        ClientBuilder::new()
    }

    pub fn get_user(&self) -> &SquireAccount {
        &self.user
    }

    // There needs to be a method + message that fetches a listenerr from the management task

    /// Creates a local tournament, imports it, and returns the id. This tournament will be pushed
    /// to the backend server but the remote import might not be completed by the time the value is
    /// returned
    pub async fn create_tournament(&self, seed: TournamentSeed) -> TournamentId {
        self.sender
            .import(TournamentManager::new(self.user.clone(), seed))
            .await
            .unwrap()
    }

    pub async fn persist_tourn_to_backend(&self, id: TournamentId) -> BackendImportStatus {
        let Some(tourn) = self.sender.query(id, |tourn| tourn.clone()).await else {
            return BackendImportStatus::NotFound;
        };
        let res = self
            .post_request(TOURNAMENTS_ROUTE.as_str(), tourn)
            .await
            .unwrap();
        if res.status().is_success() {
            BackendImportStatus::Success
        } else {
            BackendImportStatus::AlreadyImported
        }
    }

    /// Retrieves a tournament with the given id from the backend. This tournament will not update
    /// as the backend updates its version of the tournament.
    pub async fn fetch_tournament(&self, _id: TournamentId) -> bool {
        todo!()
    }

    /// Retrieves a tournament with the given id from the backend and creates a websocket
    /// connection to receive updates from the backend.
    pub async fn sub_to_tournament(&self, id: TournamentId) -> Option<Subscriber<bool>> {
        self.sender.subscribe(id).await
    }

    async fn get_request(&self, path: &str) -> Result<Response, reqwest::Error> {
        self.client.get(format!("{}{path}", self.url)).send().await
    }

    async fn post_request<B>(&self, path: &str, body: B) -> Result<Response, reqwest::Error>
    where
        B: Serialize,
    {
        self.client
            .post(format!("{}{path}", self.url))
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
    }

    pub fn import_tourn(&self, tourn: TournamentManager) -> ImportTracker {
        self.sender.import(tourn)
    }

    pub fn remove_tourn(&self, id: TournamentId) -> UpdateTracker {
        self.sender.update(id, UpdateType::Removal)
    }

    pub fn update_tourn(&self, id: TournamentId, op: TournOp) -> UpdateTracker {
        self.sender.update(id, UpdateType::Single(Box::new(op)))
    }

    pub fn bulk_update<I>(&self, id: TournamentId, iter: I) -> UpdateTracker
    where
        I: IntoIterator<Item = TournOp>,
    {
        self.sender
            .update(id, UpdateType::Bulk(iter.into_iter().collect()))
    }

    pub fn query_tourn<F, T>(&self, id: TournamentId, query: F) -> QueryTracker<T>
    where
        F: 'static + Send + FnOnce(&TournamentManager) -> T,
        T: 'static + Send,
    {
        self.sender.query(id, query)
    }

    pub fn query_players<F, T>(&self, id: TournamentId, query: F) -> QueryTracker<T>
    where
        F: 'static + Send + FnOnce(&PlayerRegistry) -> T,
        T: 'static + Send,
    {
        self.sender.query(id, move |tourn| query(&tourn.player_reg))
    }

    pub fn query_rounds<F, T>(&self, id: TournamentId, query: F) -> QueryTracker<T>
    where
        F: 'static + Send + FnOnce(&RoundRegistry) -> T,
        T: 'static + Send,
    {
        self.sender.query(id, move |tourn| query(&tourn.round_reg))
    }

    pub async fn get_tourn_summaries(&self) -> Option<Vec<TournamentSummary>> {
        let res = self
            .get_request(&LIST_TOURNAMENTS_ROUTE.replace(["0"]))
            .await
            .ok()?;
        let data = res.text().await.ok()?;
        serde_json::from_str(&data).ok()
    }
}

impl Debug for SquireClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            client,
            url,
            user,
            server_mode,
            sender,
            ..
        } = self;
        write!(
            f,
            r#"SquireClient {{ client: {client:?}, url: {url:?}, user: {user:?}, server_mode: {server_mode:?}, sender: {sender:?} }}"#
        )
    }
}
