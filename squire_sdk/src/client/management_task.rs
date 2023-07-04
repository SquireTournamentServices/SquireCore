use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{
    future::FusedFuture,
    stream::{select_all, SelectAll, SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use squire_lib::{
    operations::{OpData, OpResult, TournOp},
    tournament::TournamentId,
};
use tokio::sync::{
    broadcast::{channel as broadcast_channel, Receiver as Subscriber, Sender as Broadcaster},
    mpsc::{error::TryRecvError, unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::{channel as oneshot, Receiver as OneshotReceiver, Sender as OneshotSender},
};
use uuid::Uuid;

use super::{
    compat::{rest, spawn_task, Websocket, WebsocketError, WebsocketMessage, WebsocketResult},
    error::ClientResult,
    import::{import_channel, ImportTracker, TournamentImport},
    query::{query_channel, QueryTracker, TournamentQuery},
    subscription::{sub_channel, SubTracker, TournamentSub},
    update::{update_channel, TournamentUpdate, UpdateTracker, UpdateType},
    OnUpdate,
};
use crate::{
    api::SUBSCRIBE_ROUTE,
    sync::{
        ClientBound, ClientBoundMessage, ClientForwardingManager, ClientOpLink, ClientSyncManager,
        OpSync, ServerBound, ServerBoundMessage, ServerOpLink, SyncForwardResp, WebSocketMessage,
    },
    tournaments::TournamentManager,
};

pub const MANAGEMENT_PANICKED_MSG: &str = "tournament management task panicked";

#[derive(Debug)]
pub(crate) enum ManagementCommand {
    Query(TournamentQuery),
    Update(TournamentUpdate),
    Import(TournamentImport),
    Subscribe(TournamentSub),
}

/// A container for the channels used to communicate with the tournament management task.
#[derive(Debug, Clone)]
pub struct ManagementTaskSender {
    sender: UnboundedSender<ManagementCommand>,
}

impl ManagementTaskSender {
    pub fn import(&self, tourn: TournamentManager) -> ImportTracker {
        let (msg, digest) = import_channel(tourn);
        // FIXME: This "bubbles up" a panic from the management task. In theory, a new task can be
        // spawned; however, a panic should never happen
        self.sender
            .send(ManagementCommand::Import(msg))
            .expect(MANAGEMENT_PANICKED_MSG);
        digest
    }

    pub fn subscribe(&self, id: TournamentId) -> SubTracker {
        let (msg, tracker) = sub_channel(id);
        self.sender.send(ManagementCommand::Subscribe(msg)).unwrap();
        tracker
    }

    pub fn query<F, T>(&self, id: TournamentId, query: F) -> QueryTracker<T>
    where
        F: 'static + Send + FnOnce(&TournamentManager) -> T,
        T: 'static + Send,
    {
        let (msg, digest) = query_channel(id, query);
        // FIXME: This "bubbles up" a panic from the management task. In theory, a new task can be
        // spawned; however, a panic should never happen
        self.sender
            .send(ManagementCommand::Query(msg))
            .expect(MANAGEMENT_PANICKED_MSG);
        digest
    }

    pub fn update(&self, id: TournamentId, update: UpdateType) -> UpdateTracker {
        let (msg, digest) = update_channel(id, update);
        // FIXME: This "bubbles up" a panic from the management task. In theory, a new task can be
        // spawned; however, a panic should never happen
        self.sender
            .send(ManagementCommand::Update(msg))
            .expect(MANAGEMENT_PANICKED_MSG);
        digest
    }
}

/// Spawns a new tournament management tokio task. Communication with this task is done via a
/// collection of channels. This collection is returned
pub(super) fn spawn_management_task<F: OnUpdate>(on_update: F) -> ManagementTaskSender {
    let (send, recv) = unbounded_channel();
    // Spawn the task that will manage the tournaments and run forever
    spawn_task(tournament_management_task(recv, on_update));
    ManagementTaskSender { sender: send }
}

/// Contains all the info needed to track a tournament and all outbound communication related to
/// it. Since not all tournaments have associated outbound communicate, the `comm` field is
/// optional.
struct TournComm {
    tourn: TournamentManager,
    comm: Option<(SplitSink<Websocket, WebsocketMessage>, Broadcaster<bool>)>,
}

/// A struct that contains all of the state that the management task maintains
#[derive(Default)]
struct ManagerState {
    cache: TournamentCache,
    listener: SelectAll<SplitStream<Websocket>>,
    syncs: ClientSyncManager,
    forwarded: ClientForwardingManager,
}

type TournamentCache = HashMap<TournamentId, TournComm>;

const HANG_UP_MESSAGE: &str = "The client has been dropped.";

/// The function that manages all the tournaments for a client and runs forever inside the tokio
/// task.
///
/// FIXME: Currently, this task has no way to send outbound requests, but it will need that
/// ability. The client internals should be moved here.
async fn tournament_management_task<F>(
    mut recv: UnboundedReceiver<ManagementCommand>,
    mut on_update: F,
) where
    F: FnMut(TournamentId),
{
    let mut state = ManagerState::default();
    loop {
        let opt_sub: Option<_> = tokio::select! {
            msg = state.listener.next(), if !state.listener.is_empty() => {
                match msg {
                    Some(Ok(msg)) => handle_ws_msg(&mut state, &mut on_update, msg).await,
                    Some(Err(err)) => handle_ws_err(&mut state, err),
                    None => {}
                }
                None
            }
            msg = recv.recv() => {
                match msg.expect(HANG_UP_MESSAGE) {
                    ManagementCommand::Query(query) => {
                        handle_query(&state, query);
                        None
                    },
                    ManagementCommand::Import(import) => {
                        handle_import(&mut state, import);
                        None
                    },
                    ManagementCommand::Update(update) => {
                        handle_update(&mut state, update, &mut on_update).await;
                            None
                    }
                    ManagementCommand::Subscribe(sub) => Some(sub),
                }
            }
            (id, msg) = state.syncs.retry() => {
                match state.cache.get_mut(&id).and_then(|tourn| tourn.comm.as_mut()) {
                    Some(comm) => {
                        state.syncs.update_timer(&msg.id);
                        let bytes = WebsocketMessage::Bytes(postcard::to_allocvec(&msg).unwrap());
                        let _ = comm.0.send(bytes).await;
                    },
                    None => {
                        state.syncs.finalize_chain(&msg.id);
                    }
                };
                None
            }
        };
        if let Some(sub) = opt_sub {
            handle_sub(&mut state, sub).await
        }
    }
}

fn handle_import(state: &mut ManagerState, import: TournamentImport) {
    let TournamentImport { tourn, tracker } = import;
    let id = tourn.id;
    let tc = TournComm { tourn, comm: None };
    state.cache.insert(id, tc);
    let _ = tracker.send(id);
}

async fn handle_update<F>(state: &mut ManagerState, update: TournamentUpdate, on_update: &mut F)
where
    F: FnMut(TournamentId),
{
    let TournamentUpdate {
        local,
        remote,
        id,
        update,
    } = update;
    let mut to_remove = false;
    if let Some(tourn) = state.cache.get_mut(&id) {
        let res = match update {
            UpdateType::Single(op) => tourn.tourn.apply_op(op),
            UpdateType::Bulk(ops) => tourn.tourn.bulk_apply_ops(ops),
            UpdateType::Removal => {
                to_remove = true;
                Ok(OpData::Nothing)
            }
        };
        let is_ok = res.is_ok();
        let _ = local.send(Some(res));
        // TODO: This need to inform the cache manager that an update the be backend needs to
        // go out.
        let _ = remote.send(Some(Ok(())));
        if is_ok {
            on_update(id);
        }
        if is_ok && !to_remove {
            let id = Uuid::new_v4();
            let sync: ClientOpLink = tourn.tourn.sync_request().into();
            state
                .syncs
                .initialize_chain(id, tourn.tourn.id, sync.clone())
                .unwrap(); // TODO: Remove unwrap
            let msg = ServerBoundMessage {
                id,
                body: sync.into(),
            };
            tourn.send(msg).await;
        }
    } else {
        let _ = local.send(None);
        let _ = remote.send(None);
    }
    if to_remove {
        // This has to exist, but we don't need to use it
        let _ = state.cache.remove(&id);
    }
}

fn handle_query(state: &ManagerState, query: TournamentQuery) {
    let TournamentQuery { query, id } = query;
    query(state.cache.get(&id).map(|tc| &tc.tourn));
}

// Needs to take a &mut to the SelectAll WS listener so it can be updated if need be
async fn handle_sub(state: &mut ManagerState, TournamentSub { send, id }: TournamentSub) {
    match state.cache.entry(id) {
        Entry::Occupied(mut entry) => match &mut entry.get_mut().comm {
            // Tournament is cached and communication is set up for it
            Some((_, broad)) => {
                let _ = send.send(Some(broad.subscribe()));
            }
            // Tournament is cached but there is no communication for it
            None => {
                let url = format!(
                    "ws://localhost:8000{}",
                    SUBSCRIBE_ROUTE.replace([id.to_string().as_str()])
                );
                match create_ws_connection(&url).await {
                    Ok(ws) => {
                        let (sink, stream) = ws.split();
                        let (broad, _) = broadcast_channel(10);
                        entry.get_mut().comm = Some((sink, broad));
                        state.listener.push(stream);
                    }
                    Err(_) => {
                        let _ = send.send(None);
                        panic!()
                    }
                }
            }
        },
        // Tournament is not cached
        Entry::Vacant(entry) => {
            let url = format!(
                "ws://localhost:8000{}",
                SUBSCRIBE_ROUTE.replace([id.to_string().as_str()])
            );
            match create_ws_connection(&url).await {
                Ok(ws) => {
                    let (mut sink, mut stream) = ws.split();
                    let msg = postcard::to_allocvec(&ServerBoundMessage::new(ServerBound::Fetch))
                        .unwrap();
                    sink.send(WebsocketMessage::Bytes(msg)).await.unwrap();
                    let tourn = wait_for_tourn(&mut stream).await;
                    let (broad, _) = broadcast_channel(10);
                    let tc = TournComm {
                        tourn,
                        comm: Some((sink, broad)),
                    };
                    let _ = entry.insert(tc);
                    state.listener.push(stream);
                }
                Err(_) => {
                    let _ = send.send(None);
                    panic!()
                }
            }
        }
    }
}

async fn handle_ws_msg<F>(state: &mut ManagerState, on_update: &mut F, msg: WebsocketMessage)
where
    F: FnMut(TournamentId),
{
    let WebsocketMessage::Bytes(data) = msg else {
        panic!("Server did not send bytes of Websocket")
    };
    let WebSocketMessage { body, id } = postcard::from_bytes::<ClientBoundMessage>(&data).unwrap();
    match body {
        ClientBound::FetchResp(_) => { /* Do nothing, handled elsewhere */ }
        ClientBound::SyncChain(link) => {
            handle_server_op_link(state, on_update, &id, link).await;
        }
        ClientBound::SyncForward((t_id, sync)) => {
            handle_forwarded_sync(state, on_update, &t_id, id, sync).await
        }
    }
}

fn handle_ws_err(state: &mut ManagerState, err: WebsocketError) {
    panic!("Got error from Websocket: {err:?}")
}

async fn handle_server_op_link<F>(
    state: &mut ManagerState,
    on_update: &mut F,
    msg_id: &Uuid,
    link: ServerOpLink,
) where
    F: FnMut(TournamentId),
{
    // Get tourn
    let Some(t_id) = state.syncs.get_tourn_id(msg_id) else {
        return;
    };
    let Some(tourn) = state.cache.get_mut(&t_id) else {
        return;
    };
    match link {
        ServerOpLink::Conflict(proc) => {
            let server = ServerOpLink::Conflict(proc.clone());
            // TODO: This, somehow, needs to be a user decision...
            let dec: ClientOpLink = proc.purge().into();
            // Send decision to backend
            state.syncs.progress_chain(msg_id, dec.clone(), server);
            let msg = ServerBoundMessage {
                id: *msg_id,
                body: dec.into(),
            };
            tourn.send(msg).await;
        }
        ServerOpLink::Completed(comp) => {
            tourn.tourn.handle_completion(comp).unwrap();
            state.syncs.finalize_chain(msg_id);
            on_update(t_id);
        }
        ServerOpLink::Error(_) | ServerOpLink::TerminatedSeen { .. } => {
            state.syncs.finalize_chain(msg_id);
        }
    }
}

async fn handle_forwarded_sync<F>(
    state: &mut ManagerState,
    on_update: &mut F,
    t_id: &TournamentId,
    msg_id: Uuid,
    sync: OpSync,
) where
    F: FnMut(TournamentId),
{
    let Some(comm) = state.cache.get_mut(t_id) else {
        return;
    };
    let resp = if state.forwarded.contains_resp(&msg_id) {
        state.forwarded.get_resp(&msg_id).unwrap()
    } else {
        let resp = comm.tourn.handle_forwarded_sync(sync);
        if matches!(resp, SyncForwardResp::Success) {
            on_update(*t_id);
        }
        state.forwarded.add_resp(msg_id, resp.clone());
        resp
    };
    state.forwarded.clean();
    let msg = ServerBoundMessage {
        id: msg_id,
        body: resp.into(),
    };
    comm.send(msg).await;
}

// TODO: Add retries
async fn create_ws_connection(url: &str) -> Result<Websocket, ()> {
    Websocket::new(url).await
}

async fn wait_for_tourn(stream: &mut SplitStream<Websocket>) -> TournamentManager {
    loop {
        let Some(Ok(WebsocketMessage::Bytes(msg))) = stream.next().await else {
            continue;
        };
        let ClientBoundMessage { body, .. } = postcard::from_bytes(&msg).unwrap();
        let ClientBound::FetchResp(tourn) = body else {
            panic!("Server did not return a tournament")
        };
        return *tourn;
    }
}

impl ManagerState {
    fn new() -> Self {
        Self::default()
    }
}

impl TournComm {
    async fn send(&mut self, msg: ServerBoundMessage) {
        if let Some(comm) = self.comm.as_mut() {
            let bytes = WebsocketMessage::Bytes(postcard::to_allocvec(&msg).unwrap());
            let _ = comm.0.send(bytes).await;
        }
    }
}
