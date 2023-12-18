use std::marker::PhantomData;

use reqwest::{header::HeaderName, Method, Request};
use serde::de::DeserializeOwned;
use squire_lib::{operations::OpResult, tournament::TournRole};
use tokio::sync::watch::Receiver as Subscriber;

use self::{
    builder::ClientBuilder,
    network::NetworkClient,
    session::SessionWatcher,
    tournaments::{TournsClient, UpdateType},
};
use crate::{
    actor::Tracker,
    api::{
        Credentials, GetRequest, ListTournaments, PostRequest, RegForm, SessionToken,
        TournamentSummary,
    },
    compat::{NetworkResponse, Sendable},
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

// This needs to be `'static + Send` because of constraints on `async_trait`. Ideally, it would
// just be `Sendable`.
pub trait OnUpdate: 'static + Send + FnMut(TournamentId) {}

impl<T> OnUpdate for T where T: 'static + Send + FnMut(TournamentId) {}

pub mod builder;
pub mod error;
pub mod network;
pub mod session;
pub mod tournaments;

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

#[derive(Debug)]
pub struct SquireClient {
    user: SessionWatcher,
    client: NetworkClient,
    tourns: TournsClient,
}

pub enum BackendImportStatus {
    /// The tournament was successfully sent to the backend and stored in the database.
    Success,
    /// The tournament was sent to the backend, but the backend around had a copy of it.
    AlreadyImported,
    /// The tournament was not found locally, so it could not be persisted.
    NotFound,
}

pub struct ResponseTracker<R>(Tracker<NetworkResponse>, PhantomData<R>);

impl<R> ResponseTracker<R>
where
    R: DeserializeOwned,
{
    pub fn new(tracker: Tracker<NetworkResponse>) -> Self {
        Self(tracker, PhantomData)
    }

    pub async fn output(self) -> Result<R, reqwest::Error> {
        self.0.await.inner()?.json().await
    }
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
        let user = self.user.session_info().get_user()?;
        Some(
            self.tourns
                .import(TournamentManager::new(user.clone(), seed))
                .await,
        )
    }

    pub async fn persist_tourn_to_backend(&self, id: TournamentId) -> BackendImportStatus {
        let Some(tourn) = self.tourns.query(id, |tourn| tourn.clone()).await else {
            return BackendImportStatus::NotFound;
        };

        if self.post_request(tourn, []).output().await.unwrap() {
            BackendImportStatus::Success
        } else {
            BackendImportStatus::AlreadyImported
        }
    }

    /// Retrieves a tournament with the given id from the backend and creates a websocket
    /// connection to receive updates from the backend.
    pub async fn sub_to_tournament(&self, id: TournamentId) -> Option<Subscriber<()>> {
        self.tourns.subscribe(id).await
    }

    fn get_request<const N: usize, R>(&self, subs: [&str; N]) -> ResponseTracker<R::Response>
    where
        R: 'static + GetRequest<N>,
        R::Response: Sendable,
    {
        let url = format!("http{HOST_ADDRESS}{}", R::ROUTE.replace(subs));
        let url = reqwest::Url::parse(&url).unwrap();
        let req = Request::new(Method::GET, url);
        let tracker = self.client.track(req);
        ResponseTracker::new(tracker)
    }

    fn post_request<const N: usize, B>(
        &self,
        body: B,
        subs: [&str; N],
    ) -> ResponseTracker<B::Response>
    where
        B: Sendable + Sync + PostRequest<N>,
        B::Response: Sendable,
    {
        let url = format!("http{HOST_ADDRESS}{}", B::ROUTE.replace(subs));
        let url = reqwest::Url::parse(&url).unwrap();
        let mut req = Request::new(Method::POST, url);
        let body = serde_json::to_string(&body).unwrap();
        let _ = req.body_mut().insert(body.into());
        let _ = req.headers_mut().insert(
            HeaderName::from_static("content-type"),
            reqwest::header::HeaderValue::from_str("application/json").unwrap(),
        );
        let tracker = self.client.track(req);
        ResponseTracker::new(tracker)
    }

    pub fn import_tourn(&self, tourn: TournamentManager) -> Tracker<TournamentId> {
        self.tourns.import(tourn)
    }

    pub fn remove_tourn(&self, id: TournamentId) -> Tracker<Option<OpResult>> {
        self.tourns.update(id, UpdateType::Removal)
    }

    pub fn update_tourn(&self, id: TournamentId, op: TournOp) -> Tracker<Option<OpResult>> {
        self.tourns.update(id, UpdateType::Single(Box::new(op)))
    }

    pub fn bulk_update<I>(&self, id: TournamentId, iter: I) -> Tracker<Option<OpResult>>
    where
        I: IntoIterator<Item = TournOp>,
    {
        self.tourns
            .update(id, UpdateType::Bulk(iter.into_iter().collect()))
    }

    pub fn query_tourn<F, T>(&self, id: TournamentId, query: F) -> Tracker<Option<T>>
    where
        F: 'static + Send + FnOnce(&TournamentManager) -> T,
        T: 'static + Send,
    {
        self.tourns.query(id, query)
    }

    pub fn query_players<F, T>(&self, id: TournamentId, query: F) -> Tracker<Option<T>>
    where
        F: 'static + Send + FnOnce(&PlayerRegistry) -> T,
        T: 'static + Send,
    {
        self.tourns.query(id, move |tourn| query(&tourn.player_reg))
    }

    pub fn query_rounds<F, T>(&self, id: TournamentId, query: F) -> Tracker<Option<T>>
    where
        F: 'static + Send + FnOnce(&RoundRegistry) -> T,
        T: 'static + Send,
    {
        self.tourns.query(id, move |tourn| query(&tourn.round_reg))
    }

    pub fn register(&self, body: RegForm) -> ResponseTracker<bool> {
        self.post_request(body, [])
    }

    pub fn login(&self, cred: Credentials) -> Tracker<SessionWatcher> {
        self.client.track(cred)
    }

    pub fn guest_login(&self) -> Tracker<SessionWatcher> {
        self.client.track(())
    }

    pub async fn get_tourn_summaries(&self) -> Option<Vec<TournamentSummary>> {
        self.get_request::<1, ListTournaments>(["0"])
            .output()
            .await
            .ok()
    }

    pub async fn get_tourn_role(&self, id: TournamentId) -> TournRole {
        match self.user.session_info() {
            session::SessionInfo::Unknown | session::SessionInfo::Guest => TournRole::default(),
            session::SessionInfo::User(user) | session::SessionInfo::AuthUser(user) => {
                let u_id = *user.id;
                self.tourns
                    .query_or_default(id, move |tourn| tourn.user_role(u_id))
                    .await
            }
        }
    }

    pub fn get_user(&self) -> Option<SquireAccount> {
        self.user.session_query(|s| s.get_user())
    }
}
