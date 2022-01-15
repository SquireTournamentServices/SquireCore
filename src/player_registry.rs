use uuid::Uuid;

use crate::player::Player;

pub enum PlayerIdentifier {
    Id(Uuid),
    Name(String),
}

pub struct PlayerRegistry {
    players: Vec<Player>,
}

impl PlayerRegistry {
    pub fn new() -> Self {
        PlayerRegistry {
            players: Vec::new(),
        }
    }

    pub fn get_player_id(&self, ident: &PlayerIdentifier) -> Result<Uuid, ()> {
        match ident {
            PlayerIdentifier::Id(id) => Ok(id),
            PlayerIdentifier::Name(name) => {
                let ids: Vec<Uuid> = self.players.iter().filter(|p| p.name == name).map(|p| p.uuid.clone()).collect();
                if ids.len() != 1 {
                    Err(())
                } else {
                    Ok(ids[0])
                }
            }
        }
    }

    pub fn verify_identifier(&self, ident: &PlayerIdentifier) -> bool {
        match ident {
            PlayerIdentifier::Id(id) => self.players.iter().any(|p| p.uuid == *id),
            PlayerIdentifier::Name(name) => self.players.iter().any(|p| p.name == *name),
        }
    }
}
