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
    time::{Duration, Instant},
};

use ulid::Ulid;

use crate::sync::SyncError;

use super::{ClientOpLink, ServerBoundMessage, ServerOpLink, SyncChain};

const TO_CLEAR_TIME_LIMIT: Duration = Duration::from_secs(10);
const RETRY_TIMER: Duration = Duration::from_millis(250);

/// Tracks messages chains on the server side used during the syncing process.
#[derive(Debug, Default)]
pub struct ServerSyncManager {
    sync_chains: HashMap<Ulid, SyncChain>,
    completed_syncs: HashMap<Ulid, (ClientOpLink, ServerOpLink)>,
    /// After a message chain is completed, it is removed from the in-process map to the completed
    /// map. Completed messages need to stick around for some time since messages can be lost in
    /// transit. To know when a completed message should be cleared, we track the last time that it
    /// was it was used. When a message is used, its tracker is removed from this queue and
    /// reinserted with the current time. This maintains the ordering of the queue, oldest in the
    /// front and newest in the back.
    to_clear: VecDeque<(Ulid, Instant)>,
}

#[derive(Debug, Default)]
pub struct ClientSyncManager {
    queued: Option<ClientSyncManagerInner>,
}

#[derive(Debug)]
struct ClientSyncManagerInner {
    id: Ulid,
    current: ClientOpLink,
    chain: SyncChain,
    last_updated: Instant,
}

impl ServerSyncManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_sync_message(
        &mut self,
        id: &Ulid,
        msg: &ClientOpLink,
    ) -> Result<(), Result<ServerOpLink, SyncError>> {
        if let Some(chain) = self.sync_chains.get(id) {
            return chain.validate_client_message(&msg);
        }
        if let Some((client, server)) = self.completed_syncs.get(&id) {
            if client == msg {
                let digest = Err(Ok(server.clone()));
                self.update_to_clear(id);
                return digest;
            }
            return Err(Err(SyncError::AlreadyCompleted));
        }
        let chain = SyncChain::new(msg).map_err(Err)?;
        self.sync_chains.insert(*id, chain);
        Ok(())
    }

    pub fn add_sync_link(&mut self, id: Ulid, client: ClientOpLink, server: ServerOpLink) {
        let Some(chain) = self.sync_chains.get_mut(&id) else { return };
        let Some(comp) = chain.add_link(client, server) else { return };
        self.sync_chains.remove(&id);
        self.completed_syncs.insert(id, comp);
        self.to_clear.push_back((id, Instant::now()));
        self.clear();
    }

    fn clear(&mut self) {
        while let Some(timer) = self.to_clear.front() && timer.1.elapsed() >= TO_CLEAR_TIME_LIMIT {
            self.to_clear.pop_front();
        }
    }

    fn update_to_clear(&mut self, msg_id: &Ulid) {
        let Some(index) = self.to_clear.iter().enumerate().find_map(|(i, (id, _))| (msg_id == id).then_some(i)) else { return };
        let Some(mut msg) = self.to_clear.remove(index) else { return };
        msg.1 = Instant::now();
        self.to_clear.push_back(msg);
    }
}

impl ClientSyncManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_server_msg(&self, id: &Ulid) -> bool {
        self.queued
            .as_ref()
            .map(|inner| &inner.id == id)
            .unwrap_or_default()
    }

    pub fn initialize_chain(&mut self, id: Ulid, msg: ClientOpLink) -> Result<(), SyncError> {
        let inner = ClientSyncManagerInner::new(id, msg)?;
        self.queued.insert(inner);
        Ok(())
    }

    pub fn progress_chain(&mut self, id: &Ulid, client: ClientOpLink, server: ServerOpLink) {
        if let Some(inner) = self.queued.as_mut() && &inner.id == id && inner.progress(client, server) {
            self.finalize_chain();
        }
    }

    pub fn finalize_chain(&mut self) {
        self.queued.take();
    }

    pub fn retry<'a>(&'a self) -> MessageRetry<'a> {
        MessageRetry {
            inner: self.queued.as_ref(),
        }
    }
}

impl ClientSyncManagerInner {
    pub fn new(id: Ulid, current: ClientOpLink) -> Result<Self, SyncError> {
        let chain = SyncChain::new(&current)?;
        let last_updated = Instant::now();
        Ok(Self {
            id,
            current,
            chain,
            last_updated,
        })
    }

    /// Progresses the chain. Return if the chain is complete or not.
    pub fn progress(&mut self, mut client: ClientOpLink, server: ServerOpLink) -> bool {
        std::mem::swap(&mut self.current, &mut client);
        self.last_updated = Instant::now();
        self.chain.add_link(client, server).is_some()
    }
}

pub struct MessageRetry<'a> {
    inner: Option<&'a ClientSyncManagerInner>,
}

impl Future for MessageRetry<'_> {
    type Output = ServerBoundMessage;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.as_ref() {
            Some(inner) if inner.last_updated.elapsed() >= RETRY_TIMER => {
                let msg = ServerBoundMessage {
                    id: inner.id,
                    body: inner.current.clone().into(),
                };
                Poll::Ready(msg)
            }
            _ => Poll::Pending,
        }
    }
}
