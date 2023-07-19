use reqwest::{Client, StatusCode};
use squire_lib::accounts::SquireAccount;

use super::{error::ClientError, management_task::spawn_management_task, OnUpdate, SquireClient};
use crate::{
    api::VERSION_ROUTE,
    version::{ServerMode, Version},
};

/// A builder for the SquireClient. This builder is generic over most of its fields. This is used
/// to gate access to the build methods, requiring all necessary fields are filled before
/// construction of the client can occur.
#[derive(Debug)]
pub struct ClientBuilder<UP = Box<dyn OnUpdate>, URL = (), USER = ()> {
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
            on_update: Box::new(drop),
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<UP: OnUpdate, URL, USER> ClientBuilder<UP, URL, USER> {
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
    pub fn on_update<F: OnUpdate>(self, on_update: F) -> ClientBuilder<F, URL, USER> {
        let ClientBuilder { url, user, .. } = self;
        ClientBuilder {
            url,
            user,
            on_update,
        }
    }
}

impl<UP: OnUpdate> ClientBuilder<UP, String, SquireAccount> {
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
            client,
            url,
            user,
            server_mode,
            sender,
        })
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
            client: Client::new(),
            server_mode: ServerMode::Extended,
            url,
            user,
            sender,
        }
    }
}
