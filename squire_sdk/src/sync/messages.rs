use serde::{Deserialize, Serialize};
use squire_lib::tournament::TournamentId;
use ulid::Ulid;

use super::{OpSync, TournamentManager};

pub type ServerBoundMessage = WebSocketMessage<ServerBound>;
pub type ClientBoundMessage = WebSocketMessage<ServerBound>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WebSocketMessage<B> {
    /// The transaction id used to group requests/responses
    pub id: Ulid,
    /// The main payload being send to the receiver
    pub body: B
}

impl<B> WebSocketMessage<B> {
    pub fn new(body: B) -> Self {
        Self {
            id: Ulid::new(),
            body,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServerBound {
    /// Asks the server to send back a copy of the tournament manager for the tournament
    Fetch(TournamentId),
    /// New operations have been sent from a client and need to be merged
    SyncReq(OpSync),
    /// A sync request was forwarded to clients. This communicates that the client received and
    /// processed the request
    SyncSeen,
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientBound {
    /// Contains a copy of the tournament manager (sent in response to `ServerBound::Fetch`)
    FetchResp(TournamentManager),
    /// The result of trying to process the sync request
    SyncResp(()),
    /// The server has successfully synced with a client. This forwards those changes
    SyncForward(()),
}
