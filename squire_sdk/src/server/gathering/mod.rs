use std::collections::HashMap;

use async_trait::async_trait;
use axum::extract::ws::WebSocket;
use derive_more::From;
use futures::StreamExt;
use squire_lib::{identifiers::SquireAccountId, tournament::TournamentId};
use tokio::sync::{mpsc::Sender, oneshot::Sender as OneshotSender};
use uuid::Uuid;

use crate::{
    actor::{ActorState, Scheduler},
    api::AuthUser,
    sync::{
        processor::{SyncCompletion, SyncDecision},
        ClientBound, ClientBoundMessage, ClientOpLink, ForwardingRetry, OpSync, ServerBound,
        ServerBoundMessage, ServerForwardingManager, ServerOpLink, ServerSyncManager, SyncError,
        SyncForwardResp, TournamentManager,
    },
};

mod hall;
mod onlooker;
pub use hall::*;
pub use onlooker::*;

use super::session::SessionWatcher;

/// A message sent to a `Gathering` that subscribes a new `Onlooker`.
#[derive(Debug)]
pub enum GatheringMessage {
    GetTournament(OneshotSender<Box<TournamentManager>>),
    NewConnection(SessionWatcher, WebSocket),
    WebsocketMessage(CrierMessage),
    ResendMessage(Box<(AuthUser, ClientBoundMessage)>),
}

impl From<((), OneshotSender<Box<TournamentManager>>)> for GatheringMessage {
    fn from(((), send): ((), OneshotSender<Box<TournamentManager>>)) -> Self {
        Self::GetTournament(send)
    }
}

/// A message that communicates to the `GatheringHall` that it needs to backup tournament data.
/// How this data is backed up depends on the server implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistReadyMessage(TournamentId);

#[derive(Debug, From)]
pub enum PersistMessage {
    Get(TournamentId, OneshotSender<Option<Box<TournamentManager>>>),
    Persist(Box<TournamentManager>),
}

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
#[derive(Debug)]
pub struct Gathering {
    tourn: TournamentManager,
    onlookers: HashMap<AuthUser, Onlooker>,
    persist: Sender<PersistReadyMessage>,
    syncs: ServerSyncManager,
    forwarding: ServerForwardingManager,
}

// Send forwarding message
// Queue resend
// Either:
//  - Recv resp from client
//  - Send queued resend
//
// Need to track which chains have terminated

#[async_trait]
impl ActorState for Gathering {
    type Message = GatheringMessage;

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            GatheringMessage::GetTournament(send) => {
                send.send(Box::new(self.tourn.clone())).unwrap()
            }
            GatheringMessage::NewConnection(session, ws) => {
                let (sink, stream) = ws.split();
                let onlooker = Onlooker::new(sink);
                // If we get a session watcher that is not valid, we ignore it.
                if let Some(user) = session.auth_user() {
                    match self.onlookers.get_mut(&user) {
                        Some(ol) => *ol = onlooker,
                        None => {
                            _ = self.onlookers.insert(user.clone(), onlooker);
                        }
                    }
                    scheduler.add_stream(Crier::new(stream, user.clone(), session));
                }
            }
            GatheringMessage::WebsocketMessage(msg) => {
                self.process_websocket_message(scheduler, msg).await
            }
            GatheringMessage::ResendMessage(retry) => match self.onlookers.get_mut(&retry.0) {
                Some(onlooker) => {
                    let (user, msg) = *retry;
                    if !self.forwarding.is_terminated(&msg.id) {
                        let _ = onlooker.send_msg(&msg).await;
                        let fut = ForwardingRetry::new(user, msg);
                        scheduler.add_task(fut);
                    }
                }
                None => {
                    self.forwarding.terminate_chain(&retry.1.id);
                }
            },
        }
    }
}

impl Gathering {
    fn new(tourn: TournamentManager, persist: Sender<PersistReadyMessage>) -> Self {
        let count = tourn.tourn().get_player_count();
        Self {
            tourn,
            onlookers: HashMap::with_capacity(count),
            persist,
            syncs: ServerSyncManager::default(),
            forwarding: ServerForwardingManager::new(),
        }
    }

    fn send_persist_message(&mut self) {
        // If the persistance queue is full, we continue on
        let _persist_fut = self.persist.send(PersistReadyMessage(self.tourn.id));
    }

    async fn process_websocket_message(
        &mut self,
        scheduler: &mut Scheduler<Self>,
        msg: CrierMessage,
    ) {
        match msg {
            CrierMessage::NoAuthMessage(user, bytes) => {
                self.process_unauth_message(user, bytes).await
            }
            CrierMessage::AuthMessage(user, bytes) => {
                self.process_incoming_message(scheduler, user, bytes).await
            }
            CrierMessage::ClosingFrame(user) => drop(self.onlookers.remove(&user)),
        }
    }

    async fn process_unauth_message(&mut self, user: AuthUser, bytes: Vec<u8>) {
        let Ok(ServerBoundMessage { id, .. }) = postcard::from_bytes(&bytes) else {
            // TODO: Send a 'failed to deserialize message' to sender?
            return;
        };
        self.send_reply(user, id, SyncError::Unauthorized).await;
    }

    // TODO: Return a "real" value
    async fn process_incoming_message(
        &mut self,
        scheduler: &mut Scheduler<Self>,
        user: AuthUser,
        bytes: Vec<u8>,
    ) {
        let Ok(ServerBoundMessage { id, body }) = postcard::from_bytes(&bytes) else {
            // TODO: Send a 'failed to deserialize message' to sender?
            return;
        };
        match body {
            ServerBound::Fetch => {
                self.send_message(user, self.tourn.clone()).await;
            }
            ServerBound::SyncChain(sync) => {
                match &user {
                    // If the user is a guest, we reject the message since guests do not have the
                    // credentials to update tournaments.
                    AuthUser::Guest(_) => self.send_reply(user, id, SyncError::Unauthorized).await,
                    AuthUser::User(u_id) => {
                        // TODO: Check that the user is allowed to send the given update
                        let link = self.handle_sync_request(id, *u_id, sync);
                        // If completed, send forwarding requests
                        if let ServerOpLink::Completed(comp) = &link {
                            self.send_persist_message();
                            self.send_forwarding(scheduler, &user, comp).await;
                        }
                        self.send_reply(user, id, link).await;
                    }
                }
            }
            ServerBound::ForwardResp(resp) => self.handle_forwarding_resp(&id, resp),
        }
    }

    /// Checks that validitity of the sync msg (both in the sync manager and against the user's
    /// account info), processes the sync, updates the manager, and returns a response.
    fn handle_sync_request(
        &mut self,
        id: Uuid,
        u_id: SquireAccountId,
        link: ClientOpLink,
    ) -> ServerOpLink {
        if let Err(link) = self.syncs.validate_sync_message(&id, &link) {
            return link;
        }
        match link.clone() {
            ClientOpLink::Init(sync) => {
                // Check to make sure that the user is allowed to send these operations
                if let Err(err) = self.validate_sync_request(u_id, &sync) {
                    return err.into();
                }
                // Process the init
                let proc = match self.tourn.init_sync(sync) {
                    Ok(proc) => proc,
                    Err(err) => return ServerOpLink::Error(err),
                };
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
                if let Err(err) = self.tourn.handle_completion(comp.clone()) {
                    return ServerOpLink::Error(err);
                }
                // Return resp
                comp.into()
            }
            ClientOpLink::Terminated => {
                let already_done = self.syncs.terminate_chain(&id);
                ServerOpLink::TerminatedSeen { already_done }
            }
        }
    }

    async fn send_message<C: Into<ClientBound>>(&mut self, user: AuthUser, msg: C) {
        let msg = ClientBoundMessage::new(msg.into());
        self.send_message_inner(user, msg).await;
    }

    async fn send_reply<C: Into<ClientBound>>(&mut self, user: AuthUser, id: Uuid, msg: C) {
        let msg = ClientBoundMessage {
            id,
            body: msg.into(),
        };
        self.send_message_inner(user, msg).await;
    }

    async fn send_message_inner(&mut self, id: AuthUser, msg: ClientBoundMessage) {
        if let Some(user) = self.onlookers.get_mut(&id) {
            let _ = user.send_msg(&msg).await;
        }
    }

    async fn send_forwarding(
        &mut self,
        scheduler: &mut Scheduler<Self>,
        user: &AuthUser,
        comp: &SyncCompletion,
    ) {
        let (seed, owner) = self.tourn.seed_and_creator();
        let sync = OpSync {
            owner,
            seed,
            ops: comp.clone().as_slice(),
        };
        let msg = ClientBoundMessage::new((self.tourn.id, sync.clone()).into());
        for (id, onlooker) in self.onlookers.iter_mut().filter(|on| on.0 != user) {
            self.forwarding
                .add_msg(msg.id, id.clone(), self.tourn.id, sync.clone());
            let _ = onlooker.send_msg(&msg).await;
            let fut = ForwardingRetry::new(user.clone(), msg.clone());
            scheduler.add_task(fut);
        }
    }

    fn validate_sync_request(
        &mut self,
        id: SquireAccountId,
        sync: &OpSync,
    ) -> Result<(), SyncError> {
        let role = self.tourn.tourn().user_role(*id);
        if sync.iter().all(|op| op.op.valid_op(role)) {
            Ok(())
        } else {
            Err(SyncError::Unauthorized)
        }
    }

    fn handle_forwarding_resp(&mut self, id: &Uuid, _: SyncForwardResp) {
        self.forwarding.terminate_chain(id);
    }
}

impl From<CrierMessage> for GatheringMessage {
    fn from(value: CrierMessage) -> Self {
        Self::WebsocketMessage(value)
    }
}

impl From<(AuthUser, ClientBoundMessage)> for GatheringMessage {
    fn from((user, msg): (AuthUser, ClientBoundMessage)) -> Self {
        Self::ResendMessage(Box::new((user, msg)))
    }
}
