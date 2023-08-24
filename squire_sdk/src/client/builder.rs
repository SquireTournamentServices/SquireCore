use reqwest::{Client, StatusCode};
use squire_lib::accounts::SquireAccount;

use super::{
    error::ClientError, management_task::spawn_management_task, OnUpdate, SquireClient, UserInfo,
};
use crate::api::{
    Credentials, GetRequest, GetVersion, GuestSession, Login, PostRequest, ServerMode,
    SessionToken, Version,
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
    pub fn account_login(self, user: Credentials) -> ClientBuilder<UP, URL, Credentials> {
        let ClientBuilder { url, on_update, .. } = self;
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

impl<UP: OnUpdate> ClientBuilder<UP, String, ()> {
    /// Attempts to create a client. Construction will fail if a Squire server can not be reached
    /// using the given URL or a guest session can not be gotten from the server.
    pub async fn guest_build(self) -> Result<SquireClient, ClientError> {
        let ClientBuilder { url, on_update, .. } = self;
        let client = Client::builder().build()?;
        let server_mode = get_server_mode(&client, &url).await?;
        let (session, _) = get_session(&client, &url, &GuestSession).await?;
        let user = UserInfo::Guest(session);
        let sender = spawn_management_task(on_update);
        Ok(SquireClient {
            client,
            url,
            user,
            server_mode,
            sender,
        })
    }

    /// Creates a client but does not check if the URL is valid.
    pub fn guest_build_unchecked(self) -> SquireClient {
        let ClientBuilder { url, on_update, .. } = self;
        let user = UserInfo::Unknown;
        let sender = spawn_management_task(on_update);
        SquireClient {
            client: Client::new(),
            server_mode: ServerMode::Extended,
            url,
            sender,
            user,
        }
    }
}

impl<UP: OnUpdate> ClientBuilder<UP, String, Credentials> {
    /// Attempts to create a client. Construction will fail if a Squire server can not be reached
    /// using the given URL or if the login credentials are not valid.
    pub async fn build(self) -> Result<SquireClient, ClientError> {
        let ClientBuilder {
            url,
            user,
            on_update,
        } = self;
        let client = Client::builder().build()?;
        let server_mode = get_server_mode(&client, &url).await?;
        let (session, account) = get_session(&client, &url, &Login(user)).await?;
        let user = UserInfo::AuthUser { account, session };
        let sender = spawn_management_task(on_update);
        Ok(SquireClient {
            client,
            url,
            user,
            server_mode,
            sender,
        })
    }
}

impl<UP: OnUpdate> ClientBuilder<UP, String, SquireAccount> {
    /// Attempts to create a client. Construction will fail if a Squire server can not be reached
    /// using the given URL.
    pub async fn build(self) -> Result<SquireClient, ClientError> {
        let ClientBuilder {
            url,
            user,
            on_update,
        } = self;
        let user = UserInfo::User(user);
        let client = Client::builder().build()?;
        let server_mode = get_server_mode(&client, &url).await?;
        let sender = spawn_management_task(on_update);
        Ok(SquireClient {
            client,
            url,
            user,
            server_mode,
            sender,
        })
    }

    /// Creates a client but does not check if the URL is valid.
    pub fn build_unchecked(self) -> SquireClient {
        let ClientBuilder {
            url,
            user,
            on_update,
        } = self;
        let user = UserInfo::User(user);
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

async fn get_server_mode(client: &Client, url: &str) -> Result<ServerMode, ClientError> {
    let resp = client
        .get(format!("{url}{}", <GetVersion as GetRequest<0>>::ROUTE))
        .send()
        .await?;
    if resp.status() != StatusCode::OK {
        return Err(ClientError::FailedToConnect);
    }
    Ok(resp.json::<Version>().await?.mode)
}

async fn get_session<B>(
    client: &Client,
    url: &str,
    body: &B,
) -> Result<(SessionToken, B::Response), ClientError>
where
    B: PostRequest<0>,
{
    let resp = client
        .post(format!("{url}{}", B::ROUTE))
        .body(serde_json::to_string(body).unwrap())
        .send()
        .await?;
    if resp.status() != StatusCode::OK {
        return Err(ClientError::FailedToConnect);
    }
    let session = resp.headers().try_into().unwrap();
    Ok((session, resp.json::<B::Response>().await?))
}
