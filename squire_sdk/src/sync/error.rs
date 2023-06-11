use serde::{Deserialize, Serialize};
use squire_lib::{accounts::SquireAccount, tournament::TournamentSeed};

use super::OpId;

/// An enum that captures errors with the validity of sync requests.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// During the syncing process, the tournament is not locked and can receive updates. If an
    /// update occurs, the client must re-initialize the sync process.
    TournUpdated,
    /// At least one of the logs was empty
    EmptySync,
    /// The starting operation of the slice in unknown to the other log
    UnknownOperation(OpId),
    /// The `OpSync` was a mismatch for the tournament manager (e.g. wrong account or seed)
    InvalidRequest(RequestError),
}

/// This struct encodes a pair of objects that ought to match but don't.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Disagreement<T> {
    pub known: T,
    pub given: T,
}

/// This type encode the errors that can occur when checking the validity of a sync message
/// received by the tournament.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RequestError {
    /// The sync message was meant for a tournament with a different seed than the one specified in
    /// the sync message.
    WrongSeed(Disagreement<TournamentSeed>),
    /// The sync message was meant for a tournament with a different creator than the one specified
    /// in the sync message.
    WrongAccount(Disagreement<SquireAccount>),
    /// The sync message grew in size. The number of yet-to-be sync operations between the client
    /// and server is strictly decreasing. The client must be made aware of this. The backend
    /// tracks this through a `HashSet<OpId>`. If it sees an operation that don't fit, this error
    /// is raised.
    ///
    /// NOTE: This is either a bug in the client or server implementation or a malcisious/malformed
    /// client. In either case, the backend needs to log such problems.
    OpCountIncreased(OpId),
}

impl<T> Disagreement<T> {
    pub fn new(known: T, given: T) -> Self {
        Self { known, given }
    }
}
