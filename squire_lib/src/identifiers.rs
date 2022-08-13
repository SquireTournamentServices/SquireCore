use std::{hash::Hash, marker::PhantomData, ops::Deref};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use accounts::{SquireAccount, OrganizationAccount};

use crate::{
    accounts::{OrganizationAccount, SquireAccount},
    admin::{Admin, Judge},
    operations::FullOp,
    player::Player,
    round::Round,
    tournament::Tournament,
};

#[derive(Serialize, Deserialize, Debug)]
#[repr(C)]
/// A generic type-checked wrapper around a Uuid (to reduce boilerplate and redudent code)
pub struct TypeId<T>(pub Uuid, PhantomData<T>);

/// A type-checked Uuid for players
pub type PlayerId = TypeId<Player>;
/// A type-checked Uuid for rounds
pub type RoundId = TypeId<Round>;
/// A type-checked Uuid for tournaments
pub type TournamentId = TypeId<Tournament>;
/// A type-checked Uuid for user accounts
pub type UserAccountId = TypeId<SquireAccount>;
/// A type-checked Uuid for org accounts
pub type OrganizationAccountId = TypeId<OrganizationAccount>;
/// A type-checked Uuid for tournament operations
pub type OpId = TypeId<FullOp>;
/// A type-checked Uuid for tournament judges
pub type JudgeId = TypeId<Judge>;
/// A type-checked Uuid for tournament admin
pub type AdminId = TypeId<Admin>;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
/// An enum for identifying a player
pub enum PlayerIdentifier {
    /// The player's id
    Id(PlayerId),
    /// The player's name
    Name(String),
}

#[derive(Serialize, Deserialize, Hash, Debug, PartialEq, Eq, Clone)]
/// An enum for identifying a round
pub enum RoundIdentifier {
    /// The round's id
    Id(RoundId),
    /// The round's match number
    Number(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
/// An enum for identifying a tournament
pub enum TournamentIdentifier {
    /// The tournament's id
    Id(TournamentId),
    /// The tournament's name
    Name(String),
}

impl<T> TypeId<T> {
    /// Creates a new typed id from a Uuid
    pub fn new(id: Uuid) -> Self {
        Self(id, PhantomData)
    }
}

impl<T> Clone for TypeId<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
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

impl<T> Deref for TypeId<T> {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<TypeId<T>> for Uuid {
    fn from(other: TypeId<T>) -> Uuid {
        other.0
    }
}

impl<T> From<Uuid> for TypeId<T> {
    fn from(other: Uuid) -> TypeId<T> {
        TypeId(other, PhantomData)
    }
}

impl From<PlayerId> for PlayerIdentifier {
    fn from(other: PlayerId) -> PlayerIdentifier {
        PlayerIdentifier::Id(other)
    }
}

impl From<RoundId> for RoundIdentifier {
    fn from(other: RoundId) -> RoundIdentifier {
        RoundIdentifier::Id(other)
    }
}

impl From<TournamentId> for TournamentIdentifier {
    fn from(other: TournamentId) -> TournamentIdentifier {
        TournamentIdentifier::Id(other)
    }
}
