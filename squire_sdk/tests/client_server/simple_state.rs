use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use squire_sdk::{
    model::identifiers::TournamentId, sync::TournamentManager,
};

#[derive(Debug, Default, Clone)]
pub struct SimpleState {
    tourns: Arc<RwLock<HashMap<TournamentId, TournamentManager>>>,
}

impl SimpleState {
    pub fn new() -> Self {
        Self {
            tourns: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ClientState for SimpleState {
    fn query_tournament<Q, R>(&self, id: &TournamentId, query: Q) -> Option<R>
    where
        Q: FnOnce(&TournamentManager) -> R,
    {
        self.tourns.read().unwrap().get(id).map(query)
    }

    fn import_tournament(&self, tourn: TournamentManager) {
        let id = tourn.id;
        self.tourns.write().unwrap().insert(id, tourn);
    }
}
