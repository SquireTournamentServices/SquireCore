use serde::{Deserialize, Serialize};
use ulid::Ulid;

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
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientBound {
    /// The result of trying to process the sync request
    SyncResp(()),
    /// The server has successfully synced with a client. This forwards those changes
    SyncForward(()),
}
