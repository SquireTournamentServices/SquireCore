#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};
    use uuid::Uuid;

    use squire_lib::{
        accounts::{SharingPermissions, SquireAccount},
        identifiers::UserAccountId,
        pairings::PairingSystem,
        player_registry::PlayerRegistry,
        round_registry::RoundRegistry,
        settings::SwissPairingsSetting,
        standard_scoring::StandardScoring,
        tournament::TournamentPreset,
    };

    fn spoof_account() -> SquireAccount {
        let id: UserAccountId = Uuid::new_v4().into();
        SquireAccount {
            user_name: id.to_string(),
            display_name: id.to_string(),
            gamer_tags: HashMap::new(),
            user_id: id,
            permissions: SharingPermissions::Everything,
        }
    }

    fn spoof_data(
        count: usize,
    ) -> (
        PairingSystem,
        PlayerRegistry,
        RoundRegistry,
        StandardScoring,
    ) {
        let mut plyrs = PlayerRegistry::new();
        for _ in 0..count {
            let _ = plyrs.add_player(spoof_account());
        }

        let mut sys = PairingSystem::new(TournamentPreset::Swiss);
        sys.match_size = 4;
        (
            sys,
            plyrs,
            RoundRegistry::new(0, Duration::from_secs(0)),
            StandardScoring::new(),
        )
    }

    #[test]
    fn check_ins_function() {
        let (mut sys, plyrs, rnds, _) = spoof_data(4);
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        for id in plyrs.players.keys() {
            assert!(sys.ready_to_pair(&plyrs, &rnds));
            sys.ready_player(*id);
        }
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        for id in plyrs.players.keys() {
            sys.unready_player(*id);
            assert!(sys.ready_to_pair(&plyrs, &rnds));
        }

        //
        let (mut sys, plyrs, rnds, _) = spoof_data(4);
        sys.update_setting(SwissPairingsSetting::DoCheckIns(true).into())
            .unwrap();
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        for id in plyrs.players.keys() {
            assert!(!sys.ready_to_pair(&plyrs, &rnds));
            sys.ready_player(*id);
        }
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        for id in plyrs.players.keys() {
            sys.unready_player(*id);
            assert!(!sys.ready_to_pair(&plyrs, &rnds));
        }
    }

    #[test]
    fn simple_pair_all() {
        let (mut sys, plyrs, mut rnds, standings) = spoof_data(4);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        println!("{:?}", standings.get_standings(&plyrs, &rnds));
        // Pairings should exist
        let pairings = sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .unwrap();
        println!("{pairings:?}");
        // There should be exactly one pairing (with 4 players) and no one else
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0].len(), 4);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        let id = rnds.create_round();
        for p in &pairings.paired[0] {
            rnds.add_player_to_round(&id, *p).unwrap();
        }
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        assert!(sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .is_none());
    }
}
