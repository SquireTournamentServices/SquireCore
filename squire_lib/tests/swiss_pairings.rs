#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};
    use uuid::Uuid;

    use squire_lib::{
        accounts::{SharingPermissions, SquireAccount},
        identifiers::{PlayerId, UserAccountId},
        pairings::PairingSystem,
        player_registry::PlayerRegistry,
        round::RoundResult,
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

    #[test]
    fn simple_multi_round() {
        let (mut sys, plyrs, mut rnds, standings) = spoof_data(16);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        // Pairings should exist
        let pairings = sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .unwrap();
        println!("{pairings:?}");
        // There should be exactly 4 pods
        assert_eq!(pairings.paired.len(), 4);
        assert_eq!(pairings.paired[0].len(), 4);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        let winners: Vec<PlayerId> = pairings.paired.iter().map(|p| p[0]).collect();
        for pairing in &pairings.paired {
            let r_id = rnds.create_round();
            for p in pairing {
                rnds.add_player_to_round(&r_id, *p).unwrap();
            }
        }
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        assert!(sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .is_none());
        for (num, winner) in winners.iter().enumerate() {
            assert!(rnds
                .rounds
                .get_mut(&(num as u64))
                .unwrap()
                .record_result(RoundResult::Wins(*winner, 1))
                .is_ok());
            assert_eq!(
                rnds.rounds
                    .get_mut(&(num as u64))
                    .unwrap()
                    .winner
                    .as_ref()
                    .unwrap(),
                winner
            )
        }
        for (num, pairing) in pairings.paired.iter().enumerate() {
            assert!(rnds.rounds.get(&(num as u64)).unwrap().is_active());
            for plyr in pairing {
                assert!(rnds
                    .rounds
                    .get_mut(&(num as u64))
                    .unwrap()
                    .confirm_round((**plyr).into())
                    .is_ok());
            }
            println!("\n{:?}", rnds.rounds.get(&(num as u64)).unwrap());
            assert!(rnds.rounds.get(&(num as u64)).unwrap().is_certified());
        }
        // Rounds are all certified, let's repair
        let pairings = sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .unwrap();
        assert_eq!(pairings.paired.len(), 4);
        assert_eq!(pairings.paired[0].len(), 4);
        assert_eq!(pairings.rejected.len(), 0);
        // The first pairing should be the winners from the last round
        for plyr in winners.iter() {
            assert!(pairings.paired[0].iter().find(|p| *p == plyr).is_some());
        }
        println!("Standings: {:?}", standings.get_standings(&plyrs, &rnds));
    }

    #[test]
    fn large_multi_round() {
        let (mut sys, plyrs, mut rnds, standings) = spoof_data(200);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let mut count = 0;
        let goal = 15;
        // Pairings should exist
        let mut pairings = sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .unwrap();
        sys.repair_tolerance = 0;
        while count < goal && pairings.rejected.len() < 3 {
            count += 1;
            println!("The current count is {count}");
            let winners: Vec<PlayerId> = pairings.paired.iter().map(|p| p[0]).collect();
            let mut match_nums: Vec<u64> = Vec::with_capacity(winners.len());
            for pairing in &pairings.paired {
                let r_id = rnds.create_round();
                match_nums.push(rnds.get_round_number(&r_id).unwrap());
                for p in pairing {
                    rnds.add_player_to_round(&r_id, *p).unwrap();
                }
            }
            for (winner, num) in winners.iter().zip(match_nums.iter()) {
                assert!(rnds
                    .rounds
                    .get_mut(num)
                    .unwrap()
                    .record_result(RoundResult::Wins(*winner, 1))
                    .is_ok());
                assert_eq!(
                    rnds.rounds.get_mut(num).unwrap().winner.as_ref().unwrap(),
                    winner
                )
            }
            for (pairing, num) in pairings.paired.iter().zip(match_nums.iter()) {
                assert!(rnds.rounds.get(num).unwrap().is_active());
                for plyr in pairing {
                    assert!(rnds
                        .rounds
                        .get_mut(num)
                        .unwrap()
                        .confirm_round((**plyr).into())
                        .is_ok());
                }
                assert!(rnds.rounds.get(num).unwrap().is_certified());
            }
            pairings = sys
                .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
                .unwrap();
        }
        println!("The number of byes is: {}", pairings.rejected.len());
        assert_eq!(count, goal);
    }
}
