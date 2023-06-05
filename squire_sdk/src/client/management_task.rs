use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    future::Future,
    time::Duration, pin::Pin, task::{Context, Poll},
};

use futures::{
    stream::{select_all, SelectAll, SplitSink, SplitStream},
    SinkExt, StreamExt, future::FusedFuture,
};
use tokio::sync::{
    broadcast::{channel as broadcast_channel, Receiver as Subscriber, Sender as Broadcaster},
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender, error::TryRecvError},
    oneshot::{
        channel as oneshot, Receiver as OneshotReceiver,
        Sender as OneshotSender,
    },
};

use squire_lib::{
    operations::{OpData, OpResult, TournOp},
    tournament::TournamentId,
};

use crate::{
    api::SUBSCRIBE_ENDPOINT,
    sync::{ClientBoundMessage, ServerBound, ServerBoundMessage, WebSocketMessage},
    tournaments::TournamentManager,
};

use super::{
    compat::{rest, spawn_task, Websocket, WebsocketError, WebsocketMessage, WebsocketResult},
    error::ClientResult,
    import::{import_channel, ImportTracker, TournamentImport},
    query::{query_channel, QueryTracker, TournamentQuery},
    subscription::{TournamentSub, SubTracker, sub_channel},
    update::{update_channel, TournamentUpdate, UpdateTracker, UpdateType},
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
pub(super) fn spawn_management_task<F>(on_update: F) -> ManagementTaskSender
where
    F: 'static + Send + FnMut(),
{
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
    F: FnMut(),
{
    let mut state = ManagerState::default();
    loop {
        let opt_sub: Option<_> = tokio::select! {
            msg = state.listener.next(), if !state.listener.is_empty() => {
                match msg {
                    Some(Ok(msg)) => handle_ws_msg(&mut state, msg),
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
                        handle_update(&mut state, update, &mut on_update);
                            None
                    }
                    ManagementCommand::Subscribe(sub) => Some(sub),
                }
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

fn handle_update<F>(state: &mut ManagerState, update: TournamentUpdate, on_update: &mut F)
where
    F: FnMut(),
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
            on_update();
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
                match create_ws_connection(&SUBSCRIBE_ENDPOINT.replace([id.to_string().as_str()]))
                    .await
                {
                    Ok(ws) => {
                        let (sink, stream) = ws.split();
                        let (broad, _) = broadcast_channel(10);
                        entry.get_mut().comm = Some((sink, broad));
                        state.listener.push(stream);
                    }
                    Err(_) => {
                        let _ = send.send(None);
                    }
                }
            }
        },
        // Tournament is not cached
        Entry::Vacant(entry) => {
            match create_ws_connection(&SUBSCRIBE_ENDPOINT.replace([id.to_string().as_str()])).await
            {
                Ok(ws) => {
                    let (mut sink, mut stream) = ws.split();
                    let msg =
                        postcard::to_allocvec(&ServerBoundMessage::new(ServerBound::Fetch(id)))
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
                }
            }
        }
    }
}

fn handle_ws_msg(state: &mut ManagerState, msg: WebsocketMessage) {
    let WebsocketMessage::Bytes(data) = msg else { panic!("Server did not send bytes of Websocket") };
    let WebSocketMessage { body, .. } = postcard::from_bytes::<ClientBoundMessage>(&data).unwrap();
    match body {
        // Do nothing. This is handled elsewhere
        ServerBound::Fetch(_) => {}
        ServerBound::SyncReq(_) => todo!(),
        ServerBound::SyncSeen => todo!(),
    }
}

fn handle_ws_err(state: &mut ManagerState, err: WebsocketError) {
    panic!("Got error from Websocket: {err:?}")
}

// TODO: Add retries
async fn create_ws_connection(url: &str) -> Result<Websocket, ()> {
    Websocket::new(url).await
}

async fn wait_for_tourn(stream: &mut SplitStream<Websocket>) -> TournamentManager {
    loop {
        let Some(Ok(WebsocketMessage::Bytes(msg))) = stream.next().await else { continue };
        let Ok(tourn) = postcard::from_bytes::<TournamentManager>(&msg) else { continue };
        return tourn;
    }
}
