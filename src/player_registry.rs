
use uuid::Uuid;

use crate::player::Player;

pub enum PlayerIdentifier {
    Id(Uuid),
    Name(String)
}

pub struct PlayerRegistry {
    players: Vec<Player>,
}

impl PlayerRegistry {
    pub fn new() -> Self {
        PlayerRegistry { players: Vec::new() }
    }
}
