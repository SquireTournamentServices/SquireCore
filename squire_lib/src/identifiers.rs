use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    str::FromStr,
};

use chrono::{DateTime, Utc};
use deterministic_hash::DeterministicHasher;
use fxhash::FxHasher64;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::{
    accounts::SquireAccount,
    admin::{Admin, Judge},
    players::Player,
    rounds::Round,
    tournament::Tournament,
};

#[inline(always)]
fn id_hasher() -> DeterministicHasher<FxHasher64> {
    DeterministicHasher::new(FxHasher64::default())
}

/// Creates an ID (of any type) from a time and a hashable value
pub fn id_from_item<T, ID>(salt: DateTime<Utc>, item: T) -> TypeId<ID>
where
    T: Hash,
{
    let mut hasher = id_hasher();
    salt.hash(&mut hasher);
    let upper = hasher.finish();
    item.hash(&mut hasher);
    let lower = hasher.finish();
    Uuid::from_u64_pair(upper, lower).into()
}

/// Creates an ID (of any type) from a time and a iterator of hashable values
pub fn id_from_list<I, T, ID>(salt: DateTime<Utc>, vals: I) -> TypeId<ID>
where
    I: Iterator<Item = T>,
    T: Hash,
{
    let mut hasher = id_hasher();
    salt.hash(&mut hasher);
    let upper = hasher.finish();
    for item in vals {
        item.hash(&mut hasher);
    }
    let lower = hasher.finish();
    Uuid::from_u64_pair(upper, lower).into()
}

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
pub type SquireAccountId = TypeId<SquireAccount>;
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

#[derive(Serialize, Deserialize, Hash, Debug, PartialEq, Eq, Clone, Copy)]
/// An enum for identifying a round
pub enum RoundIdentifier {
    /// The round's id
    Id(RoundId),
    /// The round's match number
    Number(u64),
    /// The table number of an active match
    Table(u64),
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
    fn hash<H: Hasher>(&self, state: &mut H) {
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

impl<T> FromStr for TypeId<T> {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(Into::into)
    }
}

impl<T> Display for TypeId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for RoundIdentifier {
    fn default() -> Self {
        RoundIdentifier::Number(Default::default())
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
