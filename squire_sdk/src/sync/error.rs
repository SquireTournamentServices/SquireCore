use serde::{Deserialize, Serialize};
use squire_lib::{accounts::SquireAccount, error::TournamentError, tournament::TournamentSeed};

use super::OpId;

/// An enum that captures errors with the validity of sync requests.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// During the syncing process, the tournament is not locked and can receive updates. If an
    /// update occurs, the client must re-initialize the sync process.
    TournUpdated,
    /// At least one of the logs was empty
    EmptySync,
    /// In order to start the syncing process, an initializing message needs to be sent. If it is
    /// not present, this error is returned.
    NotInitialized,
    /// Once the initializing message has been set, there is no need to send it again. If that
    /// happens, this error is returned.
    AlreadyInitialized,
    /// A message, that wasn't the last known message, has been sent after the chain has completed.
    AlreadyCompleted,
    /// The starting operation of the slice in unknown to the other log
    UnknownOperation(OpId),
    /// The `OpSync` was a mismatch for the tournament manager (e.g. wrong account or seed)
    InvalidRequest(Box<RequestError>),
    /// The user was not authorized to send the message that was sent.
    Unauthorized,
}

/// An error used in the server-initialized sync process that the client uses to signal that an
/// error has occurred during the sync process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ForwardError {
    /// At least one of the logs was empty
    EmptySync,
    /// The `OpSync` was a mismatch for the tournament manager (e.g. wrong account or seed)
    InvalidRequest(Box<RequestError>),
    /// One of the new operations has caused a tournament error.
    ///
    /// NOTE: This is likely an un-recoverable error. Either the two tournaments do not have the
    /// same history of events or the tournament is acting non-deterministically. These error need
    /// to be logged.
    TournError(Box<TournamentError>),
}

/// This struct encodes a pair of objects that ought to match but don't.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Disagreement<T> {
    /// The known value
    pub known: T,
    /// The value that didn't match the known value but should have
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
    OpCountIncreased,
    /// The sync request caused a tournament error to happen on agreed-on operations.
    ///
    /// NOTE: This is either a bug in squire_lib caused by non-deterministic operations or a
    /// malcisious/malformed client. In either case, the backend needs to log such problems.
    TournError(TournamentError),
}

impl<T> Disagreement<T> {
    pub fn new(known: T, given: T) -> Self {
        Self { known, given }
    }
}
