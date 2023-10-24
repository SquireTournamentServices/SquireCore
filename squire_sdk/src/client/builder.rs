use squire_lib::accounts::SquireAccount;

use super::{
    error::ClientError, network::NetworkState, tournaments::TournsClient, OnUpdate, SquireClient,
};
use crate::{actor::ActorBuilder, api::Credentials};

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
        let ClientBuilder { on_update, url, .. } = self;
        let state = NetworkState::new(url);
        let user = state.subscribe();
        let client = ActorBuilder::new(state).launch();
        let tourns = TournsClient::new(client.clone(), on_update);
        Ok(SquireClient {
            client,
            tourns,
            user,
        })
    }

    /// Creates a client but does not check if the URL is valid.
    pub fn guest_build_unchecked(self) -> SquireClient {
        let ClientBuilder { on_update, url, .. } = self;
        let state = NetworkState::new(url);
        let user = state.subscribe();
        let client = ActorBuilder::new(state).launch();
        let tourns = TournsClient::new(client.clone(), on_update);
        SquireClient {
            client,
            tourns,
            user,
        }
    }
}

impl<UP: OnUpdate> ClientBuilder<UP, String, Credentials> {
    /// Attempts to create a client. Construction will fail if a Squire server can not be reached
    /// using the given URL or if the login credentials are not valid.
    pub async fn build(self) -> Result<SquireClient, ClientError> {
        let ClientBuilder { on_update, url, .. } = self;
        let state = NetworkState::new(url);
        let user = state.subscribe();
        let client = ActorBuilder::new(state).launch();
        let tourns = TournsClient::new(client.clone(), on_update);
        Ok(SquireClient {
            client,
            tourns,
            user,
        })
    }
}

impl<UP: OnUpdate> ClientBuilder<UP, String, SquireAccount> {
    /// Attempts to create a client. Construction will fail if a Squire server can not be reached
    /// using the given URL.
    pub async fn build(self) -> Result<SquireClient, ClientError> {
        let ClientBuilder {
            user,
            on_update,
            url,
        } = self;
        let state = NetworkState::new_with_user(url, user);
        let user = state.subscribe();
        let client = ActorBuilder::new(state).launch();
        let tourns = TournsClient::new(client.clone(), on_update);
        Ok(SquireClient {
            client,
            tourns,
            user,
        })
    }

    /// Creates a client but does not check if the URL is valid.
    pub fn build_unchecked(self) -> SquireClient {
        let ClientBuilder {
            user,
            on_update,
            url,
        } = self;
        let state = NetworkState::new_with_user(url, user);
        let user = state.subscribe();
        let client = ActorBuilder::new(state).launch();
        let tourns = TournsClient::new(client.clone(), on_update);
        SquireClient {
            client,
            tourns,
            user,
        }
    }
}
