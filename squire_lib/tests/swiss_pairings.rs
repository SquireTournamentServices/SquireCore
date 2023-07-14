#[cfg(test)]
mod tests {
    use chrono::Utc;

    use squire_lib::{
        rounds::{RoundContext, RoundResult},
        settings::SwissPairingSetting,
    };
    use squire_tests::spoof_data;

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

        let (mut sys, plyrs, rnds, _) = spoof_data(4);
        sys.update_setting(SwissPairingSetting::DoCheckIns(true).into())
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
        let _id = rnds.create_round(
            Utc::now(),
            pairings.paired[0].clone(),
            RoundContext::Contextless,
        );
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
        let winners: Vec<_> = pairings.paired.iter().map(|p| p[0]).collect();
        let matches =
            rnds.rounds_from_pairings(Utc::now(), pairings.clone(), RoundContext::Contextless);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        assert!(sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .is_none());
        for (winner, rnd) in winners.iter().zip(matches.iter()) {
            assert!(rnds
                .rounds
                .get_mut(rnd)
                .unwrap()
                .record_result(RoundResult::Wins(*winner, 1))
                .is_ok());
            assert_eq!(rnds.rounds.get_mut(rnd).unwrap().winner.unwrap(), *winner);
        }
        for (pairing, rnd) in pairings.paired.iter().zip(matches.iter()) {
            assert!(rnds.rounds.get(rnd).unwrap().is_active());
            for plyr in pairing {
                assert!(rnds
                    .rounds
                    .get_mut(rnd)
                    .unwrap()
                    .confirm_round((**plyr).into())
                    .is_ok());
            }
            println!("\n{:?}", rnds.rounds.get(rnd).unwrap());
            assert!(rnds.rounds.get(rnd).unwrap().is_certified());
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
            assert!(pairings.paired[0].iter().any(|p| p == plyr));
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
        let mut last_opps = rnds.opponents.clone();
        // Pairings should exist
        let mut pairings = sys
            .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
            .unwrap();
        sys.common.repair_tolerance = 0;
        while count < goal && pairings.rejected.len() < 3 {
            count += 1;
            println!("The current count is {count}");
            let winners: Vec<_> = pairings.paired.iter().map(|p| p[0]).collect();
            let matches =
                rnds.rounds_from_pairings(Utc::now(), pairings.clone(), RoundContext::Contextless);
            assert!(!rnds.opponents.is_empty());
            assert!(rnds
                .opponents
                .iter()
                .map(|(id, new)| last_opps.get(id).map(|o| o.len()).unwrap_or(0) + 3 == new.len())
                .reduce(|a, b| a && b)
                .unwrap());
            last_opps = rnds.opponents.clone();
            for (winner, rnd) in winners.iter().zip(matches.iter()) {
                assert!(rnds
                    .rounds
                    .get_mut(rnd)
                    .unwrap()
                    .record_result(RoundResult::Wins(*winner, 1))
                    .is_ok());
                assert_eq!(
                    rnds.rounds.get_mut(rnd).unwrap().winner.as_ref().unwrap(),
                    winner
                )
            }
            for (pairing, rnd) in pairings.paired.iter().zip(matches.iter()) {
                assert!(rnds.rounds.get(rnd).unwrap().is_active());
                for plyr in pairing {
                    assert!(rnds
                        .rounds
                        .get_mut(rnd)
                        .unwrap()
                        .confirm_round((**plyr).into())
                        .is_ok());
                }
                assert!(rnds.rounds.get(rnd).unwrap().is_certified());
            }
            pairings = sys
                .pair(&plyrs, &rnds, standings.get_standings(&plyrs, &rnds))
                .unwrap();
        }
        println!("The number of byes is: {}", pairings.rejected.len());
        assert_eq!(count, goal);
    }
}
