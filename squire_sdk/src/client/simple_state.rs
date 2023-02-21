use std::collections::HashMap;

use crate::{
    client::ClientState,
    model::{
        identifiers::{PlayerIdentifier, RoundIdentifier, TournamentId},
        players::Player,
        rounds::Round,
    },
    sync::TournamentManager,
};

#[derive(Debug, Default, Clone)]
pub struct SimpleState {
    tourns: HashMap<TournamentId, TournamentManager>,
}

impl SimpleState {
    pub fn new() -> Self {
        Self {
            tourns: HashMap::new(),
        }
    }
}

impl ClientState for SimpleState {
    fn query_tournament<Q, R>(&self, id: &TournamentId, query: Q) -> Option<R>
    where
        Q: FnOnce(&TournamentManager) -> R,
    {
        self.tourns.get(id).map(query)
    }

    fn import_tournament(&mut self, tourn: TournamentManager) {
        let id = tourn.id;
        self.tourns.insert(id, tourn);
    }
}
