use uuid::Uuid;

use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy)]
pub enum PlayerStatus {
    SignedUp,
    Registered,
    Dropped,
    Removed,
}

pub struct Player {
    pub uuid: Uuid,
    pub name: String,
    status: PlayerStatus,
}

impl Hash for Player {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        &self.uuid.hash(state);
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        &self.uuid == &other.uuid
    }
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            uuid: Uuid::new_v4(),
            name,
            status: PlayerStatus::SignedUp,
        }
    }

    pub fn update_status(&mut self, status: PlayerStatus) -> () {
        self.status = status;
    }
}
