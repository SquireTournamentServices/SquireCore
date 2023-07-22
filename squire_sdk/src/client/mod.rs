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
    model::{
        accounts::SquireAccount, identifiers::TournamentId, operations::TournOp,
        players::PlayerRegistry, rounds::RoundRegistry, tournament::TournamentSeed,
    },
    sync::TournamentManager,
    version::ServerMode,
};

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

    #[allow(dead_code)]
    async fn get_request(&self, path: &str) -> Result<Response, reqwest::Error> {
        println!("Sending a GET request to: {}{path}", self.url);
        self.client.get(format!("{}{path}", self.url)).send().await
    }

    #[allow(dead_code)]
    async fn post_request<B>(&mut self, path: &str, body: B) -> Result<Response, reqwest::Error>
    where
        B: Serialize,
    {
        println!("Sending a POST request to: {}{path}", self.url);
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
