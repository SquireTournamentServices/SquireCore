use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::tournaments::{RefreshRequest, RollbackRequest, RefreshResult};

use super::OpSync;

pub type ServerBoundMessage = WebSocketMessage<ServerBound>;
pub type ClientBoundMessage = WebSocketMessage<ServerBound>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WebSocketMessage<B> {
    /// The transaction id used to group requests/responses
    pub id: Ulid,
    /// The main payload being send to the receiver
    pub body: B
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServerBound {
    /// New operations have been sent from a client and need to be merged
    SyncReq(OpSync),
    /// A sync request was forwarded to clients. This communicates that the client received and
    /// processed the request
    SyncSeen,
    /// A client would like to know how its tournament history differs from the current tournament
    Refresh(RefreshRequest),
    /// A client has rolled back its tournament history, which needs to be synced
    Rollback(RollbackRequest),
    /// A rollback was forwarded to clients. This communicates that the client received and
    /// processed the request
    RollbackSeen,
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientBound {
    /// The result of trying to process the sync request
    SyncResp(()),
    /// The server has successfully synced with a client. This forwards those changes
    SyncForward(()),
    /// The response to a refresh request
    Refresh(RefreshResult),
    /// A client has requested that the tournament be rolled-back. This is the server's response
    RollbackResp(()),
    /// A client has requested that the tournament be rolled-back. The request was approved and
    /// needs to be forwarded to the other clients
    RollbackForward(()),
}
