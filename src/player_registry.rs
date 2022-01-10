
use crate::player::Player;

pub struct PlayerRegistry {
    players: Vec<Player>,
}

impl PlayerRegistry {
    pub fn new() -> Self {
        PlayerRegistry { players: Vec::new() }
    }
}
