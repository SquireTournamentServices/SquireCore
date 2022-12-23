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
    Client, Response, StatusCode,
};
use serde::Serialize;
use squire_lib::{
    accounts::SquireAccount,
    identifiers::{PlayerIdentifier, RoundIdentifier, TournamentId},
    players::Player,
    rounds::Round,
    tournament::{Tournament, TournamentPreset},
    tournament_manager::TournamentManager,
};

use crate::{
    accounts::{CreateAccountRequest, CreateAccountResponse, LoginRequest},
    tournaments::CreateTournamentRequest,
};

pub mod simple_state;

pub trait ClientState: Sync {
    fn query_tournament<Q, R>(&self, id: &TournamentId, query: Q) -> Option<R>
    where
        Q: FnOnce(&TournamentManager) -> R;

    fn import_tournament(&mut self, tourn: TournamentManager);

    fn query_player<Q, R>(
        &self,
        t_id: &TournamentId,
        p_ident: &PlayerIdentifier,
        query: Q,
    ) -> Option<Option<R>>
    where
        Q: FnOnce(&Player) -> R,
    {
        self.query_tournament(t_id, |t| t.get_player(p_ident).ok().map(query))
    }

    fn query_round<Q, R>(
        &self,
        t_id: &TournamentId,
        r_ident: &RoundIdentifier,
        query: Q,
    ) -> Option<Option<R>>
    where
        Q: FnOnce(&Round) -> R,
    {
        self.query_tournament(t_id, |t| t.get_round(r_ident).ok().map(query))
    }
}

#[derive(Debug)]
pub enum ClientError {
    Reqwest(reqwest::Error),
    RequestStatus(StatusCode),
    NotLoggedIn,
    LogInFailed,
    FailedToConnect,
}

pub struct SquireClient<S> {
    client: Client,
    url: String,
    user: SquireAccount,
    session: Option<Cookie<'static>>,
    state: S,
}

impl<S> SquireClient<S>
where
    S: ClientState,
{
    /// Tries to create a client. Fails if a connection can not be made at the given URL
    pub async fn new(url: String, user: SquireAccount, state: S) -> Result<Self, ClientError> {
        let client = Client::new();
        let resp = client.get(format!("{url}/api/v1/version")).send().await?;
        if resp.status() != StatusCode::OK {
            return Err(ClientError::FailedToConnect);
        }
        Ok(Self {
            session: None,
            client,
            url,
            user,
            state,
        })
    }

    /// Creates a client and does not check if the URL is valid
    pub fn new_unchecked(url: String, user: SquireAccount, state: S) -> Self {
        Self {
            session: None,
            client: Client::new(),
            url,
            user,
            state,
        }
    }

    pub async fn create_account(&mut self) -> Result<(), ClientError> {
        let body = CreateAccountRequest {
            user_name: self.user.user_name.clone(),
            display_name: self.user.display_name.clone(),
        };
        let resp = self.post_request("/api/v1/register", body).await?;
        match resp.status() {
            StatusCode::OK => {
                let resp: CreateAccountResponse = resp.json().await?;
                self.user = resp.0;
                self.login().await
            }
            status => Err(ClientError::RequestStatus(status)),
        }
    }

    pub async fn login(&mut self) -> Result<(), ClientError> {
        let body = LoginRequest { id: self.user.id };
        let resp = self.post_request("/api/v1/login", body).await?;
        let session = resp
            .cookies()
            .find(|c| c.name() == "SESSION")
            .ok_or(ClientError::LogInFailed)?;
        let cookie = Cookie::build("SESSION", session.value().to_string()).finish();
        self.session = Some(cookie);
        Ok(())
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
            .post_request_with_cookie("/api/v1/tournaments/create", body)
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

    async fn post_request_with_cookie<B>(
        &mut self,
        path: &str,
        body: B,
    ) -> Result<Response, ClientError>
    where
        B: Serialize,
    {
        let cookie = self
            .session
            .as_ref()
            .ok_or(ClientError::NotLoggedIn)?
            .to_string();
        self.client
            .post(format!("{}{path}", self.url))
            .header(COOKIE, cookie.to_string())
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
        println!("Sending request to: {}{path}", self.url);
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
