use reqwest::{
    header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
    Client, IntoUrl, Response, StatusCode,
};

use squire_lib::accounts::SquireAccount;

use crate::{
    accounts::{CreateAccountRequest, CreateAccountResponse},
    api::{REGISTER_ACCOUNT_ROUTE, VERSION_ROUTE},
    version::{ServerMode, Version},
};

use super::{
    compat::Session, error::ClientError, management_task::spawn_management_task, SquireClient,
};

fn noop() {}

/// A builder for the SquireClient. This builder is generic over most of its fields. This is used
/// to gate access to the build methods, requiring all necessary fields are filled before
/// construction of the client can occur.
#[derive(Debug)]
pub struct ClientBuilder<UP = Box<dyn 'static + Send + FnMut()>, URL = (), USER = ()> {
    url: URL,
    user: USER,
    on_update: UP,
}

impl ClientBuilder {
    /// Creates a builder for the client with the default `on_update` function being a `noop`.
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            url: (),
            user: (),
            on_update: Box::new(noop),
        }
    }
}

impl<UP, URL, USER> ClientBuilder<UP, URL, USER>
where
    UP: 'static + Send + FnMut(),
{
    /// Adds a URL to the configuration of the client. This method is required for construction.
    /// If there was already a URL in the configuration, it is discarded
    pub fn url(self, url: String) -> ClientBuilder<UP, String, USER> {
        let ClientBuilder {
            user, on_update, ..
        } = self;
        ClientBuilder {
            url,
            user,
            on_update,
        }
    }

    /// Adds a SquireAccount to the configuration of the client. This method is required for
    /// construction. If there was already an account in the configuration, it is discarded
    pub fn account(self, user: SquireAccount) -> ClientBuilder<UP, URL, SquireAccount> {
        let ClientBuilder { url, on_update, .. } = self;
        ClientBuilder {
            url,
            user,
            on_update,
        }
    }

    /// Adds a function that is called on update to the configuration of the client.
    /// If there was already a function in the configuration, it is discarded
    pub fn on_update<F>(self, on_update: F) -> ClientBuilder<F, URL, USER>
    where
        F: 'static + Send + FnMut(),
    {
        let ClientBuilder { url, user, .. } = self;
        ClientBuilder {
            url,
            user,
            on_update,
        }
    }
}

impl<UP> ClientBuilder<UP, String, SquireAccount>
where
    UP: 'static + Send + FnMut(),
{
    /// Tries to create a client. Fails if a connection can not be made at the held URL
    pub async fn build(self) -> Result<SquireClient, ClientError> {
        let ClientBuilder {
            url,
            user,
            on_update,
        } = self;
        let client = Client::builder().build()?;
        let resp = client.get(format!("{url}{VERSION_ROUTE}")).send().await?;
        if resp.status() != StatusCode::OK {
            return Err(ClientError::FailedToConnect);
        }
        let version: Version = resp.json().await?;
        let server_mode = version.mode;
        let sender = spawn_management_task(on_update);
        Ok(SquireClient {
            session: Session::default(),
            verification: None,
            client,
            url,
            user,
            server_mode,
            sender,
        })
    }

    /// Tries to create a client and register the user for an account. Fails if a connection can
    /// not be made at the held URL or the account could not be created.
    pub async fn build_with_account_creation(
        self,
        user_name: String,
        display_name: String,
    ) -> Result<SquireClient, ClientError> {
        let client = Client::new();
        let resp = client
            .get(format!("{}{VERSION_ROUTE}", self.url))
            .send()
            .await?;
        if resp.status() != StatusCode::OK {
            return Err(ClientError::FailedToConnect);
        }
        let server_mode = resp.json().await?;
        let body = CreateAccountRequest {
            user_name,
            display_name,
        };
        let resp = client
            .post(format!("{}{REGISTER_ACCOUNT_ROUTE}", self.url))
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let resp: CreateAccountResponse = resp.json().await?;
                let user = resp.0;
                let mut digest = self.build_unchecked();
                digest.server_mode = server_mode;
                digest.login().await?;
                Ok(digest)
            }
            status => Err(ClientError::RequestStatus(status)),
        }
    }

    /// Creates a client and does not check if the URL is valid nor does it attempt to login on
    /// creation
    pub fn build_unchecked(self) -> SquireClient {
        let ClientBuilder {
            url,
            user,
            on_update,
        } = self;
        let sender = spawn_management_task(on_update);
        SquireClient {
            session: Session::default(),
            verification: None,
            client: Client::new(),
            server_mode: ServerMode::Extended,
            url,
            user,
            sender,
        }
    }
}
