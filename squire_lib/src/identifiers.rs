use std::{hash::Hash, marker::PhantomData};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{player::Player, round::Round, tournament::Tournament};

#[derive(Serialize, Deserialize, Debug)]
#[repr(C)]
pub struct TypeId<T>(pub Uuid, PhantomData<T>);

pub type PlayerId = TypeId<Player>;
pub type RoundId = TypeId<Round>;
pub type TournamentId = TypeId<Tournament>;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum PlayerIdentifier {
    Id(PlayerId),
    Name(String),
}

#[derive(Serialize, Deserialize, Hash, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum RoundIdentifier {
    Id(RoundId),
    Number(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub enum TournamentIdentifier {
    Id(TournamentId),
    Name(String),
}

impl<T> TypeId<T> {
    pub fn new(id: Uuid) -> Self {
        Self(id, PhantomData)
    }
}

impl<T> Clone for TypeId<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T> Copy for TypeId<T> {}

impl<T> Hash for TypeId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> PartialEq for TypeId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for TypeId<T> {}
