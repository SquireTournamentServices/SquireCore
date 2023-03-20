use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use squire_sdk::{
    accounts::SquireAccount,
    client::state::ClientState,
    model::tournament::TournamentSeed,
    tournaments::{Tournament, TournamentId, TournamentManager, TournamentPreset},
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
}

impl ClientState for WebState {
    fn query_tournament<Q, R>(&self, id: &TournamentId, query: Q) -> Option<R>
    where
        Q: FnOnce(&TournamentManager) -> R,
    {
        self.tourns.read().unwrap().get(id).map(query)
    }

    fn import_tournament(&self, tourn: TournamentManager) {
        web_sys::console::log_1(&format!("Importing tournament id: {}", tourn.id).into());
        self.tourns.write().unwrap().insert(tourn.id, tourn);
    }
}
