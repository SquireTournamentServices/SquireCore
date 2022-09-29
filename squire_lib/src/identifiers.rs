use std::{hash::Hash, marker::PhantomData, ops::Deref, fmt::Display};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::{
    accounts::{OrganizationAccount, SquireAccount},
    admin::{Admin, Judge},
    operations::FullOp,
    player::Player,
    round::Round,
    tournament::Tournament,
};

#[derive(Debug)]
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

impl<T> Default for TypeId<T> {
    fn default() -> Self {
        Self(Uuid::default(), PhantomData)
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

impl<'de, T> Deserialize<'de> for TypeId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Uuid::deserialize(deserializer).map(|id| id.into())
    }
}

impl<T> Serialize for TypeId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Display for TypeId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::{AdminId, PlayerId};
    use crate::admin::Admin;

    #[test]
    fn basic_serde() {
        let id = Uuid::new_v4();
        let p_id: PlayerId = id.into();
        assert_eq!(
            serde_json::to_string(&id).unwrap(),
            serde_json::to_string(&p_id).unwrap()
        );
        let new_p_id: PlayerId =
            serde_json::from_str(&serde_json::to_string(&id).unwrap()).unwrap();
        assert_eq!(id, new_p_id.0);
        assert_eq!(p_id, new_p_id);
    }

    #[test]
    fn mapped_ids_serde() {
        let mut map: HashMap<AdminId, Admin> = HashMap::new();
        let admin = Admin {
            name: "Test".into(),
            id: Uuid::new_v4().into(),
        };
        let id = admin.id;
        map.insert(id, admin);
        let data = serde_json::to_string(&map);
        assert!(data.is_ok());
        let new_map: HashMap<AdminId, Admin> = serde_json::from_str(&data.unwrap()).unwrap();
        assert_eq!(new_map, map);
    }
}
