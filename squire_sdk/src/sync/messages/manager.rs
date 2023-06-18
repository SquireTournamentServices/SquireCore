//! The sync process has different mechanics depending on if you're on the client or server side.
//!
//! From the server's perspective, the flow of sync messages is as follows:
//!  - Received and decoded
//!  - Validated
//!   - Potentially sent responded to if validation short-curcuits
//!  - Processed in the tournament manager
//!  - Copied and stored here
//!  - Returned to the client
//!
//! From the client's perspective, the flow of sync messages is as follows:
//!  - Message is queued for retries and sent
//!  - Response is received and decoded
//!  - Check that the received message matches the currently in-progress chain
//!  - Response is processed by the tournament manager
//!  - The old message is dequeued
//!  - If still in progress, this repeats with the new message that the manager provides
//!
//! The sync forwarding process looks similar but shorter and with the roles reversed.
//!
//! From the server's perspective, the flow of the forwarding process is as follows:
//!  - Message is queued for retries and sent
//!  - Response is received and decoded
//!  - It is checked that the response corresponds with an active chain
//!   - If so, that chain is closed
//!
//! From the client's perspective, the floow of the forwarding process is as follows:
//!  - Received and decoded
//!  - Processed in the tournament manager
//!  - Returned to the server
//!  - A copy is tracked as the response might be dropped in transit and the response needs to be
//!  resent

use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use instant::Instant;

use squire_lib::tournament::TournamentId;
use uuid::Uuid;

use crate::sync::SyncError;

use super::{ClientOpLink, ServerBoundMessage, ServerOpLink, SyncChain};

const TO_CLEAR_TIME_LIMIT: Duration = Duration::from_secs(10);
const RETRY_TIMER: Duration = Duration::from_millis(250);

/// Tracks messages chains on the server side used during the syncing process.
#[derive(Debug, Default)]
pub struct ServerSyncManager {
    sync_chains: HashMap<Uuid, SyncChain>,
    completed_syncs: HashMap<Uuid, (ClientOpLink, ServerOpLink)>,
    /// After a message chain is completed, it is removed from the in-process map to the completed
    /// map. Completed messages need to stick around for some time since messages can be lost in
    /// transit. To know when a completed message should be cleared, we track the last time that it
    /// was it was used. When a message is used, its tracker is removed from this queue and
    /// reinserted with the current time. This maintains the ordering of the queue, oldest in the
    /// front and newest in the back.
    to_clear: TimerStack,
}

#[derive(Debug, Default)]
pub struct ClientSyncManager {
    syncs: HashMap<Uuid, ClientSyncTracker>,
    to_retry: TimerStack,
}

#[derive(Debug)]
struct ClientSyncTracker {
    id: TournamentId,
    current: ClientOpLink,
    chain: SyncChain,
}

impl ServerSyncManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_sync_message(
        &mut self,
        id: &Uuid,
        msg: &ClientOpLink,
    ) -> Result<(), ServerOpLink> {
        if let Some(chain) = self.sync_chains.get(id) {
            return chain.validate_client_message(msg);
        }
        if let Some((client, server)) = self.completed_syncs.get(id) {
            if client == msg {
                let digest = Err(server.clone());
                self.to_clear.update_timer(id);
                return digest;
            }
            return Err(SyncError::AlreadyCompleted.into());
        }
        let chain = SyncChain::new(msg)?;
        self.sync_chains.insert(*id, chain);
        Ok(())
    }

    pub fn add_sync_link(&mut self, id: Uuid, client: ClientOpLink, server: ServerOpLink) {
        let Some(chain) = self.sync_chains.get_mut(&id) else { return };
        let Some(comp) = chain.add_link(client, server) else { return };
        self.sync_chains.remove(&id);
        self.completed_syncs.insert(id, comp);
        self.to_clear.add_timer(id);
        self.to_clear.clear(TO_CLEAR_TIME_LIMIT);
    }

    /// Removes a chain from the in-progress map but does *not* insert it into the completed map.
    /// The bool that is returned indicates if the sync had already been completed.
    pub fn terminate_chain(&mut self, id: &Uuid) -> bool {
        self.sync_chains.remove(id);
        self.to_clear.remove_timer(id);
        self.completed_syncs.contains_key(id)
    }
}

impl ClientSyncManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_tourn_id(&self, id: &Uuid) -> Option<TournamentId> {
        self.syncs.get(id).map(|inner| inner.id)
    }

    pub fn validate_server_msg(&self, id: &Uuid) -> bool {
        self.syncs.contains_key(id)
    }

    pub fn initialize_chain(
        &mut self,
        id: Uuid,
        t_id: TournamentId,
        msg: ClientOpLink,
    ) -> Result<(), SyncError> {
        let inner = ClientSyncTracker::new(t_id, msg)?;
        self.syncs.insert(id, inner);
        self.to_retry.add_timer(id);
        Ok(())
    }

    pub fn progress_chain(&mut self, id: &Uuid, client: ClientOpLink, server: ServerOpLink) {
        if let Some(inner) = self.syncs.get_mut(id) && inner.progress(client, server) {
            self.finalize_chain(id);
        }
        self.to_retry.update_timer(id);
    }

    pub fn finalize_chain(&mut self, id: &Uuid) {
        self.syncs.remove(id);
        self.to_retry.remove_timer(id);
    }

    pub fn retry(&self) -> MessageRetry<'_> {
        let inner = self
            .to_retry
            .iter()
            .find_map(|(id, timer)| self.syncs.get(id).map(|tracker| (*id, timer, tracker)));
        MessageRetry { inner }
    }
}

impl ClientSyncTracker {
    pub fn new(id: TournamentId, current: ClientOpLink) -> Result<Self, SyncError> {
        let chain = SyncChain::new(&current)?;
        Ok(Self { id, current, chain })
    }

    /// Progresses the chain. Return if the chain is complete or not.
    pub fn progress(&mut self, mut client: ClientOpLink, server: ServerOpLink) -> bool {
        std::mem::swap(&mut self.current, &mut client);
        self.chain.add_link(client, server).is_some()
    }
}

/// Tracks the next message that need to be retried.
pub struct MessageRetry<'a> {
    inner: Option<(Uuid, &'a Instant, &'a ClientSyncTracker)>,
}

impl Future for MessageRetry<'_> {
    type Output = ServerBoundMessage;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.as_ref() {
            Some(inner) if inner.1.elapsed() >= RETRY_TIMER => {
                let msg = ServerBoundMessage {
                    id: inner.0,
                    body: inner.2.current.clone().into(),
                };
                Poll::Ready(msg)
            }
            _ => Poll::Pending,
        }
    }
}

#[derive(Debug, Default)]
struct TimerStack {
    queue: VecDeque<(Uuid, Instant)>,
}

impl TimerStack {
    fn new() -> Self {
        Self::default()
    }

    fn add_timer(&mut self, id: Uuid) {
        self.queue.push_back((id, Instant::now()));
    }

    fn update_timer(&mut self, id: &Uuid) {
        let Some(mut timer) = self.remove_timer(id) else { return };
        timer.1 = Instant::now();
        self.queue.push_back(timer);
    }

    fn remove_timer(&mut self, id: &Uuid) -> Option<(Uuid, Instant)> {
        let index = self
            .queue
            .iter()
            .enumerate()
            .find_map(|(i, (timer_id, _))| (id == timer_id).then_some(i))?;
        self.queue.remove(index)
    }

    fn clear(&mut self, limit: Duration) {
        while let Some(timer) = self.queue.front() && timer.1.elapsed() >= limit {
            self.queue.pop_front();
        }
    }

    fn iter(&self) -> impl Iterator<Item = &(Uuid, Instant)> {
        self.queue.iter()
    }
}
