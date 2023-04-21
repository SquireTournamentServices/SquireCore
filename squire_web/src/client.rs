use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tokio::sync::oneshot::{channel, Receiver, Sender, self};

use squire_sdk::{
    accounts::SquireAccount,
    client::state::ClientState,
    model::tournament::TournamentSeed,
    tournaments::{
        OpResult, TournOp, Tournament, TournamentId, TournamentManager, TournamentPreset,
    },
};

#[derive(Debug)]
pub struct WebState {
    tourns: Arc<RwLock<HashMap<TournamentId, TournamentManager>>>,
}

fn spoof_account() -> SquireAccount {
    SquireAccount::new("Tester".into(), "Tester".into())
}

fn get_seed() -> TournamentSeed {
    TournamentSeed {
        name: "Some Tourn".into(),
        preset: TournamentPreset::Swiss,
        format: "Some Format".into(),
    }
}

impl WebState {
    pub fn new() -> Self {
        Self {
            tourns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_tournament(&self) -> TournamentId {
        let tourn = TournamentManager::new(spoof_account(), get_seed());
        let id = tourn.id;
        self.tourns.write().unwrap().insert(id, tourn);
        id
    }

    pub fn submit_update(&self, id: TournamentId, op: TournOp) -> UpdateTracker {
        /* Submit update message to channel, contains send half of oneshot channel
         * Return tracker
         */
        todo!()
    }

    pub fn submit_bulk_update(&self, id: TournamentId, ops: Vec<TournOp>) -> UpdateTracker {
        /* Submit update message to channel, contains send half of oneshot channels
         * Return tracker
         */
        todo!()
    }
}

impl ClientState for WebState {
    fn query_tournament<Q, R>(&self, id: &TournamentId, query: Q) -> Option<R>
    where
        Q: FnOnce(&TournamentManager) -> R,
    {
        self.tourns.read().unwrap().get(id).map(query)
    }

    fn import_tournament(&self, tourn: TournamentManager) {
        self.tourns.write().unwrap().insert(tourn.id, tourn);
    }
}

pub struct UpdateTracker {
    local: oneshot::Receiver<OpResult>,
    remote: oneshot::Receiver<ClientResult>,
}

pub struct UpdateMessage {
    local: oneshot::Sender<OpResult>,
    remote: oneshot::Sender<ClientResult>,
    id: TournamentId,
    update_type: UpdateType,
}

pub enum UpdateType {
    Single(TournOp),
    Bulk(Vec<TournOp>),
}

pub enum UpdateStatus {
    Working,
    ChangedLocally(OpResult),
    PushedRemotely(ClientResult),
    Complete(OpResult, ClientResult),
}

pub enum ClientResult {}
