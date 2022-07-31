use squire_lib::player::Player;

pub fn spoof_player() -> Player {
    Player::new(uuid::Uuid::new_v4().to_string())
}
