use crate::error::TournamentError;
use crate::tournament::{Tournament, TournamentId};

use mtgjson::model::deck::Deck;

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};

use std::{
    collections::{hash_map::Iter, HashMap},
    slice::SliceIndex,
};

#[derive(Debug, Clone)]
pub enum TournIdentifier {
    Id(TournamentId),
    Name(String),
}

pub struct TournamentRegistry {
    tourns: DashMap<TournamentId, Tournament>,
}

impl Default for TournamentRegistry {
    fn default() -> Self {
        TournamentRegistry::new()
    }
}

impl TournamentRegistry {
    pub fn new() -> Self {
        TournamentRegistry {
            tourns: DashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.tourns.len()
    }

    pub fn get_mut_tourn(
        &self,
        ident: TournIdentifier,
    ) -> Result<RefMut<TournamentId, Tournament>, TournamentError> {
        let id = self.get_tourn_id(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.tourns.get_mut(&id).unwrap())
    }

    pub fn get_tourn(&self, ident: TournIdentifier) -> Result<Ref<TournamentId, Tournament>, TournamentError> {
        let id = self.get_tourn_id(ident)?;
        // Saftey check, we just verified that the id was valid
        Ok(self.tourns.get(&id).unwrap())
    }

    pub fn get_tourn_id(&self, ident: TournIdentifier) -> Result<TournamentId, TournamentError> {
        match ident {
            TournIdentifier::Id(id) => {
                if self.verify_identifier(&TournIdentifier::Id(id)) {
                    Ok(id)
                } else {
                    Err(TournamentError::PlayerLookup)
                }
            }
            TournIdentifier::Name(name) => {
                let ids: Vec<TournamentId> = self
                    .tourns
                    .iter()
                    .filter(|i| i.value().name == name)
                    .map(|i| i.key().clone())
                    .collect();
                if ids.len() != 1 {
                    Err(TournamentError::PlayerLookup)
                } else {
                    Ok(ids[0])
                }
            }
        }
    }

    pub fn verify_identifier(&self, ident: &TournIdentifier) -> bool {
        match ident {
            TournIdentifier::Id(id) => self.tourns.contains_key(id),
            TournIdentifier::Name(name) => self.tourns.iter().any(|i| i.value().name == *name),
        }
    }
}
