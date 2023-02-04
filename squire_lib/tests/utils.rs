use std::collections::HashMap;

use squire_lib::{
    accounts::{SharingPermissions, SquireAccount},
    players::Player,
};
use uuid::Uuid;

pub fn spoof_account() -> SquireAccount {
    let id = Uuid::new_v4().into();
    SquireAccount {
        id,
        user_name: id.to_string(),
        display_name: id.to_string(),
        gamer_tags: HashMap::new(),
        permissions: SharingPermissions::Everything,
    }
}

pub fn spoof_player() -> Player {
    Player::new(uuid::Uuid::new_v4().to_string())
}
