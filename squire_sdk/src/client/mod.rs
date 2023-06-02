#![allow(
    missing_docs,
    dead_code,
    unused_variables,
    unused_imports,
    unused_import_braces,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    unreachable_pub
)]

use std::{
    collections::HashMap,
    fmt::{self, Debug},
    sync::{Arc, Mutex, RwLock},
};

use cookie::Cookie;
use reqwest::{
    header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
    Client, IntoUrl, Response, StatusCode,
};
use serde::Serialize;
use squire_lib::{
    operations::{OpResult, TournOp},
    players::PlayerRegistry,
    rounds::RoundRegistry,
    tournament::TournamentSeed,
};

use crate::{
    api::{
        GET_TOURNAMENT_ROUTE, LOGOUT_ROUTE, REGISTER_ACCOUNT_ROUTE,
        VERIFY_ACCOUNT_ROUTE, VERSION_ROUTE,
    },
    client::error::ClientError,
    model::{
        accounts::SquireAccount,
        identifiers::{PlayerIdentifier, RoundIdentifier, TournamentId},
        players::Player,
        rounds::Round,
        tournament::{Tournament, TournamentPreset},
    },
    sync::TournamentManager,
    tournaments::CreateTournamentRequest,
    version::{ServerMode, Version},
    COOKIE_NAME,
};

use self::{
    builder::ClientBuilder,
    compat::Session,
    error::ClientResult,
    import::ImportTracker,
    management_task::{spawn_management_task, ManagementTaskSender},
    query::QueryTracker,
    update::{UpdateTracker, UpdateType},
};

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
    pub fn builder() -> ClientBuilder<Box<dyn 'static + Send + FnMut()>, (), ()> {
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
        todo!()
    }

    /// Retrieves a tournament with the given id from the backend. This tournament will not update
    /// as the backend updates its version of the tournament.
    pub async fn fetch_tournament(&self, id: TournamentId) -> bool {
        todo!()
    }

    /// Retrieves a tournament with the given id from the backend and creates a websocket
    /// connection to receive updates from the backend.
    pub async fn sub_to_tournament(&self, id: TournamentId) -> bool {
        todo!()
    }

    async fn get_request(&self, path: &str) -> Result<Response, reqwest::Error> {
        println!("Sending a GET request to: {}{path}", self.url);
        self.client.get(format!("{}{path}", self.url)).send().await
    }

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
        self.sender.update(id, UpdateType::Single(op))
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
