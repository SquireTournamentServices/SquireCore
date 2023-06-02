use std::{collections::HashMap, fmt::Debug, future::Future, time::Duration};

use squire_lib::{
    operations::{OpData, OpResult, TournOp},
    tournament::TournamentId,
};

use crate::tournaments::TournamentManager;

use super::{
    compat::{rest, spawn_task, unbounded_channel, UnboundedReceiver, UnboundedSender},
    error::ClientResult,
    import::{import_channel, ImportTracker, TournamentImport},
    query::{query_channel, QueryTracker, TournamentQuery},
    update::{update_channel, TournamentUpdate, UpdateTracker, UpdateType},
};

pub const MANAGEMENT_PANICKED_MSG: &str = "tournament management task panicked";

/// A container for the channels used to communicate with the tournament management task.
#[derive(Debug, Clone)]
pub struct ManagementTaskSender {
    query: UnboundedSender<TournamentQuery>,
    update: UnboundedSender<TournamentUpdate>,
    import: UnboundedSender<TournamentImport>,
}

impl ManagementTaskSender {
    pub fn import(&self, tourn: TournamentManager) -> ImportTracker {
        let (msg, digest) = import_channel(tourn);
        // FIXME: This "bubbles up" a panic from the management task. In theory, a new task can be
        // spawned; however, a panic should never happen
        self.import.send(msg).expect(MANAGEMENT_PANICKED_MSG);
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
        self.query.send(msg).expect(MANAGEMENT_PANICKED_MSG);
        digest
    }

    pub fn update(&self, id: TournamentId, update: UpdateType) -> UpdateTracker {
        let (msg, digest) = update_channel(id, update);
        // FIXME: This "bubbles up" a panic from the management task. In theory, a new task can be
        // spawned; however, a panic should never happen
        self.update.send(msg).expect(MANAGEMENT_PANICKED_MSG);
        digest
    }
}

/// Spawns a new tournament management tokio task. Communication with this task is done via a
/// collection of channels. This collection is returned
pub(super) fn spawn_management_task<F>(on_update: F) -> ManagementTaskSender
where
    F: 'static + Send + FnMut(),
{
    let (query, query_recv) = unbounded_channel();
    let (update, update_recv) = unbounded_channel();
    let (import, import_recv) = unbounded_channel();
    // Spawn the task that will manage the tournaments and run forever
    spawn_task(tournament_management_task(
        query_recv,
        update_recv,
        import_recv,
        on_update,
    ));
    ManagementTaskSender {
        query,
        update,
        import,
    }
}

type TournamentCache = HashMap<TournamentId, TournamentManager>;

/// The function that manages all the tournaments for a client and runs forever inside the tokio
/// task.
///
/// FIXME: Currently, this task has no way to send outbound requests, but it will need that
/// ability. The client internals should be moved here.
async fn tournament_management_task<F>(
    mut queries: UnboundedReceiver<TournamentQuery>,
    mut updates: UnboundedReceiver<TournamentUpdate>,
    mut imports: UnboundedReceiver<TournamentImport>,
    mut on_update: F,
) where
    F: FnMut(),
{
    let mut cache = TournamentCache::new();
    loop {
        if let Ok(import) = imports.try_recv() {
            handle_import(&mut cache, import);
        }
        if let Ok(update) = updates.try_recv() {
            handle_update(&mut cache, update, &mut on_update);
        }
        if let Ok(query) = queries.try_recv() {
            handle_query(&cache, query);
        }
        rest(Duration::from_millis(5)).await;
    }
}

fn handle_import(cache: &mut TournamentCache, import: TournamentImport) {
    let TournamentImport { tourn, tracker } = import;
    let id = tourn.id;
    cache.insert(id, tourn);
    let _ = tracker.send(id);
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
            UpdateType::Single(op) => tourn.apply_op(op),
            UpdateType::Bulk(ops) => tourn.bulk_apply_ops(ops),
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
    query(cache.get(&id));
}
