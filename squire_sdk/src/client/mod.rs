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

use std::sync::Arc;

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
};

use crate::{
    accounts::{
        CreateAccountRequest, CreateAccountResponse, LoginRequest, VerificationData,
        VerificationRequest, VerificationResponse,
    },
    api::{
        CREATE_TOURNAMENT_ENDPOINT, GET_TOURNAMENT_ROUTE, LOGOUT_ROUTE, REGISTER_ACCOUNT_ROUTE,
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
pub mod update;

#[derive(Debug, Clone)]
pub struct SquireClient {
    client: Client,
    url: String,
    user: SquireAccount,
    session: Session,
    verification: Option<VerificationData>,
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

    pub async fn login(&mut self) -> Result<(), ClientError> {
        let body = LoginRequest { id: self.user.id };
        let resp = self.post_request(LOGOUT_ROUTE.as_str(), body).await?;
        self.store_cred(&resp)
    }

    fn store_cred(&mut self, resp: &Response) -> ClientResult<()> {
        self.session.load_from_resp(resp)
    }

    pub fn is_verify(&self) -> bool {
        self.verification
            .as_ref()
            .map(|data| data.status)
            .unwrap_or_default()
    }

    pub async fn verify(&mut self) -> Result<String, ClientError> {
        println!("Attempting to verify!");
        let data = match &self.verification {
            Some(data) => self.verify_get().await?,
            None => self.verify_post().await?,
        };
        let digest = data.confirmation.clone();
        self.verification = Some(data);
        Ok(digest)
    }

    async fn verify_post(&mut self) -> Result<VerificationData, ClientError> {
        let body = VerificationRequest {
            account: self.user.clone(),
        };
        println!("Sending verification request!");
        let resp = self
            .post_request(VERIFY_ACCOUNT_ROUTE.as_str(), body)
            .await?;
        self.store_cred(&resp)?;
        let digest: VerificationResponse = resp.json().await?;
        let digest = digest.0.map_err(|_| ClientError::LogInFailed)?;
        Ok(digest)
    }

    async fn verify_get(&mut self) -> Result<VerificationData, ClientError> {
        let resp = self
            .get_request_with_cookie(VERIFY_ACCOUNT_ROUTE.as_str())
            .await?;
        Ok(resp.json::<VerificationResponse>().await?.0.unwrap())
    }

    pub async fn create_tournament(
        &mut self,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> Result<TournamentId, ClientError> {
        let body = CreateTournamentRequest {
            name,
            preset,
            format,
        };
        let resp = self
            .post_request_with_cookie(CREATE_TOURNAMENT_ENDPOINT.as_str(), body)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let tourn: TournamentManager = resp.json().await?;
                let id = tourn.id;
                self.sender.import(tourn).process().await;
                Ok(id)
            }
            status => Err(ClientError::RequestStatus(status)),
        }
    }

    pub async fn fetch_tournament(&self, id: TournamentId) -> Result<(), ClientError> {
        let tourn = self
            .get_request(&GET_TOURNAMENT_ROUTE.replace(&[id.to_string().as_str()]))
            .await?
            .json()
            .await?;
        self.sender.import(tourn);
        Ok(())
    }

    async fn get_request_with_cookie(&self, path: &str) -> Result<Response, ClientError> {
        self.client
            .get(format!("{}{path}", self.url))
            .header(COOKIE, self.cred_string()?)
            .send()
            .await
            .map_err(Into::into)
    }

    fn cred_string(&self) -> Result<String, ClientError> {
        self.session.cred_string()
    }

    async fn get_request(&self, path: &str) -> Result<Response, reqwest::Error> {
        println!("Sending a GET request to: {}{path}", self.url);
        self.client.get(format!("{}{path}", self.url)).send().await
    }

    async fn post_request_with_cookie<B>(
        &mut self,
        path: &str,
        body: B,
    ) -> Result<Response, ClientError>
    where
        B: Serialize,
    {
        self.client
            .post(format!("{}{path}", self.url))
            .header(COOKIE, self.cred_string()?)
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
            .map_err(Into::into)
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
