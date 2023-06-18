use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::Hash,
    sync::Arc,
    time::Duration,
};

use async_session::Session;
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SelectAll, SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use squire_lib::{identifiers::SquireAccountId, tournament::TournamentId};
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot::{channel as oneshot_channel, Sender as OneshotSender},
        OnceCell,
    },
    time::Instant,
};
use ulid::Ulid;

use crate::{
    sync::{
        processor::{SyncCompletion, SyncDecision},
        ClientBound, ClientBoundMessage, ClientOpLink, OpSync, ServerBound, ServerBoundMessage,
        ServerOpLink, ServerSyncManager, SyncError, SyncForwardResp,
    },
    tournaments::TournamentManager,
};

use super::{state::ServerState, User};

mod hall;
mod onlooker;
pub use hall::*;
pub use onlooker::*;

/// A message sent to a `Gathering` that subscribes a new `Onlooker`.
#[derive(Debug)]
pub enum GatheringMessage {
    GetTournament(OneshotSender<TournamentManager>),
    NewConnection(User, WebSocket),
}

/// A message that communicates to the `GatheringHall` that it needs to backup tournament data.
/// How this data is backed up depends on the server implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistMessage(Arc<TournamentManager>);

/// This structure contains all users currently subscribed to a tournament and can be thought of as
/// the crowd of people that gathers for a tournament. New subscribers, called `Onlooker`s, are
/// added by the `GatheringHall` via a message passed through a channel. This structure is intented
/// to be consumed and turned into a tokio task. Inside that task, the Gathering listens to
/// messages coming from WebSockets, processes those messages, and the sends those messages back
/// out to all the other `Onlooker`s.
///
/// NOTE: The Gathering contains a copy of the tournament manager. This copy is the most up-to-date
/// version of the tournament. However, backup copies of the tournament are sent to `GatheringHall`
/// whenever there is a change and other locations upon request.
pub struct Gathering {
    tourn: TournamentManager,
    messages: Receiver<GatheringMessage>,
    ws_streams: SelectAll<Crier>,
    onlookers: HashMap<SquireAccountId, Onlooker>,
    persist: Sender<PersistMessage>,
    syncs: ServerSyncManager,
}

impl Gathering {
    fn new(
        tourn: TournamentManager,
        new_onlookers: Receiver<GatheringMessage>,
        persist: Sender<PersistMessage>,
    ) -> Self {
        let count = tourn.tourn().get_player_count();
        Self {
            tourn,
            messages: new_onlookers,
            ws_streams: SelectAll::new(),
            onlookers: HashMap::with_capacity(count),
            persist,
            syncs: ServerSyncManager::default(),
        }
    }

    async fn run(mut self) -> ! {
        println!("Running gathering for tournament: {}", self.tourn.id);
        loop {
            tokio::select! {
                msg = self.messages.recv() =>
                    self.process_channel_message(msg.unwrap()),
                msg = self.ws_streams.next(), if !self.ws_streams.is_empty() =>
                    self.process_websocket_message(msg).await,
            }
        }
    }

    async fn process_websocket_message(&mut self, msg: Option<(SquireAccountId, Option<Vec<u8>>)>) {
        match msg {
            Some((user, Some(bytes))) => {
                self.process_incoming_message(user, bytes).await;
            }
            Some((user, None)) => {
                self.onlookers.remove(&user);
            }
            None => {}
        }
    }

    fn process_channel_message(&mut self, msg: GatheringMessage) {
        match msg {
            GatheringMessage::GetTournament(send) => send.send(self.tourn.clone()).unwrap(),
            GatheringMessage::NewConnection(user, ws) => {
                let (sink, stream) = ws.split();
                let crier = Crier::new(stream, user.account.id);
                let onlooker = Onlooker::new(sink);
                self.ws_streams.push(crier);
                match self.onlookers.get_mut(&user.account.id) {
                    Some(ol) => *ol = onlooker,
                    None => {
                        self.onlookers.insert(user.account.id, onlooker);
                    }
                }
            }
        }
    }

    // TODO: Return a "real" value
    async fn process_incoming_message(&mut self, user: SquireAccountId, bytes: Vec<u8>) {
        let ServerBoundMessage { id, body } =
            match postcard::from_bytes::<ServerBoundMessage>(&bytes) {
                Ok(val) => val,
                Err(_) => {
                    // TODO: Send a 'failed to deserialize message' to sender
                    return;
                }
            };
        match body {
            ServerBound::Fetch => {
                self.send_message(user, self.tourn.clone()).await;
            }
            ServerBound::SyncChain(sync) => {
                let link = self.handle_sync_request(id, sync);
                // If completed, send forwarding requests
                if let ServerOpLink::Completed(comp) = &link {
                    self.send_forwarding(&user, comp).await;
                }
                self.send_reply(user, id, link).await;
            }
            ServerBound::ForwardResp(resp) => self.handle_forwarding_resp(resp),
        }
    }

    /// Checks that validitity of the sync msg (both in the sync manager and against the user's
    /// account info), processes the sync, updates the manager, and returns a response.
    fn handle_sync_request(&mut self, id: Ulid, link: ClientOpLink) -> ServerOpLink {
        if let Err(link) = self.syncs.validate_sync_message(&id, &link) {
            return link;
        }
        match link.clone() {
            ClientOpLink::Init(sync) => {
                if let Err(err) = self.validate_sync_request(&sync) {
                    return err.into();
                }
                // Process the init
                let proc = self.tourn.init_sync(sync)?;
                let resp = self.tourn.process_sync(proc);
                // Convert into a resp
                self.syncs.add_sync_link(id, link, resp.clone());
                // Return resp
                resp
            }
            ClientOpLink::Decision(SyncDecision::Plucked(proc)) => {
                // Continue to try to resolve
                let resp = self.tourn.process_sync(proc);
                // Get resp
                self.syncs.add_sync_link(id, link, resp.clone());
                // Return resp
                resp
            }
            ClientOpLink::Decision(SyncDecision::Purged(comp)) => {
                // Apply and get resp
                self.tourn.handle_completion(comp.clone())?;
                // Return resp
                comp.into()
            }
            ClientOpLink::Terminated => {
                let already_done = self.syncs.terminate_chain(&id);
                ServerOpLink::TerminatedSeen { already_done }
            }
        }
    }

    async fn send_message<C: Into<ClientBound>>(&mut self, id: SquireAccountId, msg: C) {
        let msg = ClientBoundMessage::new(msg.into());
        self.send_message_inner(id, msg).await;
    }

    async fn send_reply<C: Into<ClientBound>>(&mut self, user: SquireAccountId, id: Ulid, msg: C) {
        let msg = ClientBoundMessage {
            id,
            body: msg.into(),
        };
        self.send_message_inner(user, msg).await;
    }

    async fn send_message_inner(&mut self, id: SquireAccountId, msg: ClientBoundMessage) {
        if let Some(user) = self.onlookers.get_mut(&id) {
            let _ = user.send_msg(&msg).await;
        }
    }

    async fn send_forwarding(&mut self, id: &SquireAccountId, comp: &SyncCompletion) {
        let (seed, owner) = self.tourn.seed_and_creator();
        let sync = OpSync {
            owner,
            seed,
            ops: comp.clone().as_slice(),
        };
        let msg = ClientBoundMessage::new(sync.into());
        for (_, onlooker) in self.onlookers.iter_mut().filter(|on| on.0 != id) {
            onlooker.send_msg(&msg).await;
        }
    }

    // TODO: Return an actual error
    // TODO: This method does not actually check to see if the person that sent the request is
    // allowed to send such a return. This will need to eventually change
    fn validate_sync_request(&mut self, sync: &OpSync) -> Result<(), SyncError> {
        Ok(())
    }

    fn handle_forwarding_resp(&mut self, resp: SyncForwardResp) {
        todo!()
    }

    fn disperse(self) {
        todo!()
    }
}