use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{OpSync, TournamentManager, SyncError};

pub type ServerBoundMessage = WebSocketMessage<ServerBound>;
pub type ClientBoundMessage = WebSocketMessage<ClientBound>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WebSocketMessage<B> {
    /// The transaction id used to group requests/responses
    pub id: Ulid,
    /// The main payload being send to the receiver
    pub body: B,
}

impl<B> WebSocketMessage<B> {
    pub fn new(body: B) -> Self {
        Self {
            id: Ulid::new(),
            body,
        }
    }

    pub fn new_with_id(id: Ulid, body: B) -> Self {
        Self { id, body }
    }

    pub fn swap_body<T>(self, new_body: T) -> (WebSocketMessage<T>, B) {
        let WebSocketMessage { id, body } = self;
        (WebSocketMessage { id, body: new_body }, body)
    }

    pub fn swap_body_with<F, T>(self, f: F) -> (WebSocketMessage<T>, B)
    where
        F: FnOnce(&mut B) -> T,
    {
        let WebSocketMessage { id, mut body } = self;
        let new_body = f(&mut body);
        (WebSocketMessage { id, body: new_body }, body)
    }

    pub fn into_resp<T>(self, body: T) -> WebSocketMessage<T> {
        let WebSocketMessage { id, .. } = self;
        WebSocketMessage { id, body }
    }

    pub fn into_resp_with<F, T>(self, f: F) -> WebSocketMessage<T>
    where
        F: FnOnce(B) -> T,
    {
        let WebSocketMessage { id, body } = self;
        WebSocketMessage { id, body: f(body) }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServerBound {
    /// Asks the server to send back a copy of the tournament manager for the tournament
    Fetch,
    /// New operations have been sent from a client and need to be merged
    SyncChain(ClientOpLink),
    SyncResp(SyncForwardResp),
}

pub enum ClientOpLink {
    Init(OpSync),
    Decision(OpDecision),
    Error(SyncError),
}

pub enum ServerOpLink {
    Conflict(OpProcessor),
    Completed(OpCompletion),
    Error(SyncError),
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientBound {
    /// Contains a copy of the tournament manager (sent in response to `ServerBound::Fetch`)
    FetchResp(TournamentManager),
    SyncChain(ServerOpLink),
    SyncForward(SyncForward),
}

pub struct SyncForward();

pub enum SyncForwardResp {
    Success,
    Aborted,
    Error(ForwardError),
}

pub enum ForwardError {
}
