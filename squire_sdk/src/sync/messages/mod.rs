use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    processor::{SyncCompletion, SyncDecision, SyncProcessor},
    ForwardError, OpSync, SyncError, TournamentManager,
};

mod chain;
mod manager;
pub use chain::*;
pub use manager::*;

pub type ServerBoundMessage = WebSocketMessage<ServerBound>;
pub type ClientBoundMessage = WebSocketMessage<ClientBound>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WebSocketMessage<B> {
    /// The transaction id used to group requests/responses
    pub id: Uuid,
    /// The main payload being send to the receiver
    pub body: B,
}

impl<B> WebSocketMessage<B> {
    pub fn new(body: B) -> Self {
        Self {
            id: Uuid::new_v4(),
            body,
        }
    }

    pub fn new_with_id(id: Uuid, body: B) -> Self {
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

/// This type encodes all of the messages that a client might send to the backend via a Websocket.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServerBound {
    /// Asks the server to send back a copy of the tournament manager for the tournament
    Fetch,
    /// The client has operations that it needs to sync with the backend. This encode a link in
    /// the chain of messages needed to sync.
    SyncChain(ClientOpLink),
    /// The backend has sent operations that need to be synced with the client. This is the
    /// client's response.
    ForwardResp(SyncForwardResp),
}

/// This type encodes all of the messages that the backend might send to a client via a Websocket.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientBound {
    /// The client has requested a copy of the tournament data. This is that copy.
    FetchResp(Box<TournamentManager>),
    /// The client has started the process of syncing tournament data with the server. This encodes
    /// the server's message in the sync message chain.
    SyncChain(ServerOpLink),
    /// The server wishes to sync with a client. This encodes the messages the backend can send in
    /// that process.
    SyncForward(OpSync),
}

/// The process of syncing two instances of a tournament (between client and server) requires a
/// series of messages to be passed back and forth. This type encodes all of the messages that a
/// client can send to the backend in this process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientOpLink {
    /// The client is asking to initialize a sync
    Init(OpSync),
    /// The sync ran into a problem, and the client has decided which operation(s) shall be removed
    /// from its log. The client wishes to try and finalize the sync.
    Decision(SyncDecision),
    /// The client wishes to terminate the sync attempt. Often, this occurs because it has received
    /// new operations that will be lumped into a new sync request.
    Terminated,
}

/// The process of syncing two instances of a tournament (between client and server) requires a
/// series of messages to be passed back and forth. This type encodes all of the messages that the
/// backend can send to a client in this process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServerOpLink {
    /// The server is process the sync request from the client, but a tournament error has occured,
    /// blocking the sync. The client must rectify this problem.
    Conflict(SyncProcessor),
    /// The server was able to complete the sync request. The server must communicate the final log
    /// (i.e. if there are new operations for the client).
    Completed(SyncCompletion),
    /// The client has requested that the sync it initialized be terminated. The backend will
    /// terminate the request, but it must communicate if the request finished before this message
    /// arrived.
    TerminatedSeen { already_done: bool },
    /// During the sync process, some kind of error occured between deserializing the message and
    /// processing the first operations (generally, this is an error with the `OpSync`). This needs
    /// to be communicated with the client. This implicitly closes the request.
    Error(SyncError),
}

/// The server has requested the client sync with it. This type encodes all of the ways that the
/// client can respond.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SyncForwardResp {
    /// The sync was received and the client did not have any operations that were unknown to the
    /// backend. The sync was processed successful.
    Success,
    /// The sync was received but the client has operations that are unknown to the backend. The
    /// server-initialized sync is cancelled, and the client will initialize another sync.
    Aborted,
    /// Some kind of error has occured. The client need to make the backend aware of this. This
    /// implicitly cancels the sync.
    Error(ForwardError),
}
