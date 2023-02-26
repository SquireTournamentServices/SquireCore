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

use cookie::Cookie;
use reqwest::{
    header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
    Client, IntoUrl, Response, StatusCode,
};
use serde::Serialize;

use crate::{
    accounts::{
        CreateAccountRequest, CreateAccountResponse, LoginRequest, VerificationData,
        VerificationRequest, VerificationResponse,
    },
    api::{
        CREATE_TOURNAMENT_ENDPOINT, LOGOUT_ROUTE, REGISTER_ACCOUNT_ROUTE, VERIFY_ACCOUNT_ROUTE,
        VERSION_ROUTE,
    },
    client::{error::ClientError, state::ClientState},
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

pub mod error;
pub mod simple_state;
pub mod state;

#[derive(Debug, Clone)]
pub struct SquireClient<S> {
    client: Client,
    url: String,
    user: SquireAccount,
    #[cfg(target_family = "wasm")]
    session: Option<()>,
    #[cfg(not(target_family = "wasm"))]
    session: Option<Cookie<'static>>,
    verification: Option<VerificationData>,
    server_mode: ServerMode,
    pub state: S,
}

impl<S> SquireClient<S>
where
    S: ClientState,
{
    /// Tries to create a client. Fails if a connection can not be made at the given URL
    pub async fn new(url: String, user: SquireAccount, state: S) -> Result<Self, ClientError> {
        let client = Client::builder().build()?;
        let resp = client.get(format!("{url}{VERSION_ROUTE}")).send().await?;
        if resp.status() != StatusCode::OK {
            return Err(ClientError::FailedToConnect);
        }
        let version: Version = resp.json().await?;
        let server_mode = version.mode;
        Ok(Self {
            session: None,
            verification: None,
            client,
            url,
            user,
            server_mode,
            state,
        })
    }

    pub async fn with_account_creation(
        url: String,
        user_name: String,
        display_name: String,
        state: S,
    ) -> Result<Self, ClientError> {
        let client = Client::new();
        let resp = client.get(format!("{url}{VERSION_ROUTE}")).send().await?;
        if resp.status() != StatusCode::OK {
            return Err(ClientError::FailedToConnect);
        }
        let server_mode = resp.json().await?;
        let body = CreateAccountRequest {
            user_name,
            display_name,
        };
        let resp = client
            .post(format!("{url}{REGISTER_ACCOUNT_ROUTE}"))
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let resp: CreateAccountResponse = resp.json().await?;
                let user = resp.0;
                let mut digest = Self::new_unchecked(url, user, state);
                digest.server_mode = server_mode;
                digest.login().await?;
                Ok(digest)
            }
            status => Err(ClientError::RequestStatus(status)),
        }
    }

    /// Creates a client and does not check if the URL is valid
    pub fn new_unchecked(url: String, user: SquireAccount, state: S) -> Self {
        Self {
            session: None,
            verification: None,
            client: Client::new(),
            server_mode: ServerMode::Extended,
            url,
            user,
            state,
        }
    }

    pub async fn login(&mut self) -> Result<(), ClientError> {
        let body = LoginRequest { id: self.user.id };
        let resp = self.post_request(LOGOUT_ROUTE.as_str(), body).await?;
        self.store_cred(&resp)
    }

    #[cfg(target_family = "wasm")]
    fn store_cred(&mut self, _: &Response) -> Result<(), ClientError> {
        // TODO: This is really all that we can do because of the browser?
        self.session = Some(());
        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    fn store_cred(&mut self, resp: &Response) -> Result<(), ClientError> {
        let session = resp
            .cookies()
            .find(|c| c.name() == COOKIE_NAME)
            .ok_or(ClientError::LogInFailed)?;
        let cookie = Cookie::build(COOKIE_NAME, session.value().to_string()).finish();
        self.session = Some(cookie);
        Ok(())
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
                self.state.import_tournament(tourn);
                Ok(id)
            }
            status => Err(ClientError::RequestStatus(status)),
        }
    }

    async fn get_request_with_cookie(&self, path: &str) -> Result<Response, ClientError> {
        self.client
            .get(format!("{}{path}", self.url))
            .header(COOKIE, self.cred_string()?)
            .send()
            .await
            .map_err(Into::into)
    }

    #[cfg(target_family = "wasm")]
    fn cred_string(&self) -> Result<String, ClientError> {
        self
            .session
            .as_ref()
            .map(|_| String::new())
            .ok_or(ClientError::NotLoggedIn)
    }

    #[cfg(not(target_family = "wasm"))]
    fn cred_string(&self) -> Result<String, ClientError> {
        self
            .session
            .as_ref()
            .map(|c| c.to_string())
            .ok_or(ClientError::NotLoggedIn)
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
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        ClientError::Reqwest(value)
    }
}
