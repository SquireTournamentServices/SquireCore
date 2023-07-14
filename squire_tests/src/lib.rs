use std::{collections::HashMap, time::Duration};

use uuid::Uuid;

mod sdk;
pub use sdk::*;
use squire_lib::{
    accounts::{SharingPermissions, SquireAccount},
    pairings::PairingSystem,
    players::{Player, PlayerRegistry},
    rounds::RoundRegistry,
    scoring::StandardScoring,
    tournament::TournamentPreset,
};
use squire_lib::tournament_seed::TournamentSeed;

pub fn get_seed() -> TournamentSeed {
    TournamentSeed::new(
        "Test Tournament".into(),
        TournamentPreset::Swiss,
        "Pioneer".into(),
    )
    .unwrap()
}

pub fn get_fluid_seed() -> TournamentSeed {
    TournamentSeed::new(
        "Test Tournament".into(),
        TournamentPreset::Fluid,
        "Pioneer".into(),
    )
    .unwrap()
}

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

pub fn spoof_data(
    count: usize,
) -> (
    PairingSystem,
    PlayerRegistry,
    RoundRegistry,
    StandardScoring,
) {
    let mut plyrs = PlayerRegistry::new();
    for _ in 0..count {
        let _ = plyrs.register_player(spoof_account());
    }

    let mut sys = PairingSystem::new(TournamentPreset::Swiss);
    sys.common.match_size = 4;
    (
        sys,
        plyrs,
        RoundRegistry::new(0, Duration::from_secs(0)),
        StandardScoring::new(),
    )
}

pub fn spoof_fluid_pairings() -> PairingSystem {
    let mut sys = PairingSystem::new(TournamentPreset::Fluid);
    sys.common.match_size = 4;
    sys
}
