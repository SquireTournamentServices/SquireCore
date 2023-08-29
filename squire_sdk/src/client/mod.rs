use std::fmt::{self, Debug};

use reqwest::{header::CONTENT_TYPE, Client};
use squire_lib::operations::OpResult;
use tokio::sync::broadcast::Receiver as Subscriber;

use self::{
    builder::ClientBuilder,
    management_task::{ManagementTaskSender, Tracker, UpdateType},
};
use crate::{
    api::{GetRequest, ListTournaments, PostRequest, ServerMode, SessionToken, TournamentSummary},
    model::{
        accounts::SquireAccount, identifiers::TournamentId, operations::TournOp,
        players::PlayerRegistry, rounds::RoundRegistry, tournament::TournamentSeed,
    },
    sync::TournamentManager,
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
pub mod management_task;

/// Encapsulates the known account and session information of the user
#[derive(Debug, Clone)]
pub enum UserInfo {
    /// No information is known about the user
    Unknown,
    /// The user has provided account information, but a session is not known
    User(SquireAccount),
    /// The user has started a guest session with the server
    Guest(SessionToken),
    /// The user has provided account information and has authenticated with the server
    AuthUser {
        account: SquireAccount,
        session: SessionToken,
    },
}

impl UserInfo {
    pub fn get_token(&self) -> Option<&SessionToken> {
        match self {
            UserInfo::Unknown | UserInfo::User(_) => None,
            UserInfo::Guest(session) | UserInfo::AuthUser { session, .. } => Some(session),
        }
    }
    pub fn get_user(&self) -> Option<&SquireAccount> {
        match self {
            UserInfo::Unknown | UserInfo::Guest(_) => None,
            UserInfo::User(account) | UserInfo::AuthUser { account, .. } => Some(account),
        }
    }
}

pub struct SquireClient {
    client: Client,
    user: UserInfo,
    url: String,
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

    // There needs to be a method + message that fetches a listener from the management task

    /// Creates a local tournament, imports it, and returns the id. This tournament will be pushed
    /// to the backend server but the remote import might not be completed by the time the value is
    /// returned
    pub async fn create_tournament(&self, seed: TournamentSeed) -> Option<TournamentId> {
        let user = self.user.get_user()?;
        Some(
            self.sender
                .import(TournamentManager::new(user.clone(), seed))
                .await,
        )
    }

    pub async fn persist_tourn_to_backend(&self, id: TournamentId) -> BackendImportStatus {
        let Some(tourn) = self.sender.query(id, |tourn| tourn.clone()).await else {
            return BackendImportStatus::NotFound;
        };
        if self.post_request(tourn, []).await.unwrap() {
            BackendImportStatus::Success
        } else {
            BackendImportStatus::AlreadyImported
        }
    }

    /// Retrieves a tournament with the given id from the backend and creates a websocket
    /// connection to receive updates from the backend.
    pub async fn sub_to_tournament(&self, id: TournamentId) -> Option<Subscriber<bool>> {
        self.sender.subscribe(id).await
    }

    async fn get_request<const N: usize, R>(
        &self,
        subs: [&str; N],
    ) -> Result<R::Response, reqwest::Error>
    where
        R: GetRequest<N>,
    {
        self.client
            .get(format!("{}{}", self.url, R::ROUTE.replace(subs)))
            .send()
            .await?
            .json()
            .await
    }

    async fn post_request<const N: usize, B>(
        &self,
        body: B,
        subs: [&str; N],
    ) -> Result<B::Response, reqwest::Error>
    where
        B: PostRequest<N>,
    {
        let mut builder = self
            .client
            .post(format!("{}{}", self.url, B::ROUTE.replace(subs)))
            .header(CONTENT_TYPE, "application/json");
        if let Some((name, header)) = self.user.get_token().map(SessionToken::as_header) {
            builder = builder.header(name, header);
        }
        builder
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await?
            .json()
            .await
    }

    pub fn import_tourn(&self, tourn: TournamentManager) -> Tracker<TournamentId> {
        self.sender.import(tourn)
    }

    pub fn remove_tourn(&self, id: TournamentId) -> Tracker<Option<OpResult>> {
        self.sender.update(id, UpdateType::Removal)
    }

    pub fn update_tourn(&self, id: TournamentId, op: TournOp) -> Tracker<Option<OpResult>> {
        self.sender.update(id, UpdateType::Single(Box::new(op)))
    }

    pub fn bulk_update<I>(&self, id: TournamentId, iter: I) -> Tracker<Option<OpResult>>
    where
        I: IntoIterator<Item = TournOp>,
    {
        self.sender
            .update(id, UpdateType::Bulk(iter.into_iter().collect()))
    }

    pub fn query_tourn<F, T>(&self, id: TournamentId, query: F) -> Tracker<Option<T>>
    where
        F: 'static + Send + FnOnce(&TournamentManager) -> T,
        T: 'static + Send,
    {
        self.sender.query(id, query)
    }

    pub fn query_players<F, T>(&self, id: TournamentId, query: F) -> Tracker<Option<T>>
    where
        F: 'static + Send + FnOnce(&PlayerRegistry) -> T,
        T: 'static + Send,
    {
        self.sender.query(id, move |tourn| query(&tourn.player_reg))
    }

    pub fn query_rounds<F, T>(&self, id: TournamentId, query: F) -> Tracker<Option<T>>
    where
        F: 'static + Send + FnOnce(&RoundRegistry) -> T,
        T: 'static + Send,
    {
        self.sender.query(id, move |tourn| query(&tourn.round_reg))
    }

    pub async fn get_tourn_summaries(&self) -> Option<Vec<TournamentSummary>> {
        self.get_request::<1, ListTournaments>(["0"]).await.ok()
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
