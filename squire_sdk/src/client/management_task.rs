use std::{collections::HashMap, fmt::Debug, future::Future, time::Duration};

use futures::stream::{SplitSink, SplitStream};
use tokio::sync::{
    broadcast::{Receiver as Subscriber, Sender as Broadcaster},
    mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel},
    oneshot::{
        channel as oneshot, error::TryRecvError, Receiver as OneshotReceiver,
        Sender as OneshotSender,
    },
};

use squire_lib::{
    operations::{OpData, OpResult, TournOp},
    tournament::TournamentId,
};

use crate::tournaments::TournamentManager;

use super::{
    compat::{rest, spawn_task, Websocket, WebsocketMessage},
    error::ClientResult,
    import::{import_channel, ImportTracker, TournamentImport},
    query::{query_channel, QueryTracker, TournamentQuery},
    subscription::TournamentSub,
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
    let mut cache = TournamentCache::new();
    loop {
        futures::select! {
            msg = recv.recv() => {
                match msg.expect(HANG_UP_MESSAGE) {
                    ManagementCommand::Query(query) => handle_query(&cache, query),
                    ManagementCommand::Import(import) => handle_import(&mut cache, import),
                    ManagementCommand::Update(update) => handle_update(&mut cache, update, &mut on_update),
                    ManagementCommand::Subscribe(sub) => handle_sub(&mut cache, sub).await,
                }
            }
        }
    }
}

fn handle_import(cache: &mut TournamentCache, import: TournamentImport) {
    let TournamentImport { tourn, tracker } = import;
    let id = tourn.id;
    //cache.insert(id, tourn);
    //let _ = tracker.send(id);
}

fn handle_update<F>(cache: &mut TournamentCache, update: TournamentUpdate, on_update: &mut F)
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
    if let Some(tourn) = cache.get_mut(&id) {
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
        let _ = cache.remove(&id);
    }
}

fn handle_query(cache: &TournamentCache, query: TournamentQuery) {
    let TournamentQuery { query, id } = query;
    query(cache.get(&id).map(|tc| &tc.tourn));
}

// Needs to take a &mut to the SelectAll WS listener so it can be updated if need be
async fn handle_sub(cache: &mut TournamentCache, TournamentSub { send, id }: TournamentSub) {
    /*
    match cache.get(id).map(|(_, broad)| broad) {
        // Tournament is cached and communication is set up for it
        Some(Some(broad)) => {
            let sub = broad;
        },
        // Tournament is cached but there is no communication for it
        Some(None) => todo!(),
        // Tournament is not cached
        None => todo!(),
    }
    */
    // Check to see if the tournament is already in the sublist
    //  - If so, return a listener
    // If not, open a connection
    // Handle the new connnection
    // Return a listener
    todo!()
}
