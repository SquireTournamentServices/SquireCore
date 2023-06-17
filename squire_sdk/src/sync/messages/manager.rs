use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use ulid::Ulid;

use super::{
    ClientBound, ClientOpLink, ForwardChain, ServerBound, ServerOpLink, SyncChain, WebSocketMessage,
};

/// Tracks messages chains used during the syncing process.
#[derive(Debug, Default)]
pub struct MessageManager {
    sync_chains: HashMap<Ulid, SyncChain>,
    forward_chains: HashMap<Ulid, ForwardChain>,
    completed_syncs: HashMap<Ulid, (ClientOpLink, ServerOpLink)>,
    completed_forwards: HashMap<Ulid, ()>,
    /// After a message chain is completed, it is removed from the in-process map to the completed
    /// map. Completed messages need to stick around for some time since messages can be lost in
    /// transit. To know when a completed message should be cleared, we track the last time that it
    /// was it was used. When a message is used, its tracker is removed from this queue and
    /// reinserted with the current time. This maintains the ordering of the queue, oldest in the
    /// front and newest in the back.
    to_clear: VecDeque<(Ulid, Instant)>,
}

impl MessageManager {
    pub fn new() -> Self {
        Self::default()
    }
}

