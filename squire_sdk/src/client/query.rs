use std::{fmt::Debug, time::Duration};

use squire_lib::tournament::TournamentId;

use crate::tournaments::TournamentManager;

use super::compat::{oneshot, OneshotReceiver};

/// The type used to send requests for arbitary calculations to the tournament management task. The
/// boxed function contains a tokio oneshot channel, which communicates the calculation results
/// back to the caller
pub(crate) struct TournamentQuery {
    pub(crate) id: TournamentId,
    #[allow(clippy::type_complexity)]
    pub(crate) query: Box<dyn Send + FnOnce(Option<&TournamentManager>)>,
}

/// A wrapper around the receiver half of the oneshot channel that is used to communicate the
/// result of the query
#[derive(Debug)]
pub struct QueryTracker<T> {
    recv: OneshotReceiver<Option<T>>,
}

impl<T> QueryTracker<T> {
    /// Consumes self and waits for the task to finish processing the query
    pub async fn process(self) -> Option<T> {
        self.recv.await.ok().flatten()
    }
}

/// This function takes queries to tournament managers and returns managed wrappers around channels
/// that handle the sending and receiving of query results. This allows arbitary calculations to be
/// done by the tournament management task and passed back to the caller.
///
/// How it works:
/// A sender/receiver oneshot channel pair is created. A closure that wraps the given query is
/// created and boxed. The wrapping closure takes an `Option<&TournamentManager>`, maps that
/// optional reference using the query, and sends the  mapped option over sender half of the channel.
/// The wrapping closure is boxed (for type easure) and returned in the `TournamentQuery`. The
/// receiver half of the channel is returned inside of the `QueryTracker`.
///
/// A caller will never interact a `TournamentQuery` direct as it is only meant to be an interface
/// between the squire client and the tournament management task. Similarly, a `QueryTracker` will
/// never be construct by the caller, but the caller will interact similar to how one would
/// interact with a oneshot channel.
///
/// NOTE: The given query returns some type `T`, but the channel returns an `Option<T>`. This
/// is because the tournament must be looked up and might not exist.
pub(crate) fn query_channel<F, T>(id: TournamentId, query: F) -> (TournamentQuery, QueryTracker<T>)
where
    F: 'static + Send + FnOnce(&TournamentManager) -> T,
    T: 'static + Send,
{
    let (send, recv) = oneshot();
    let query = Box::new(move |tourn: Option<&TournamentManager>| {
        // We ignore the result from `send` as it means the receiver was dropped and result of
        // the query is no longer needed.
        let _ = send.send(tourn.map(query));
    });
    (TournamentQuery { id, query }, QueryTracker { recv })
}

impl Debug for TournamentQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TournamentQuery {{ query... some boxed function :/ }}")
    }
}
