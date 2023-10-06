#[cfg(test)]
mod tests {
    use chrono::Utc;
    use squire_lib::{
        rounds::RoundContext,
        scoring::{StandardScore, Standings},
    };
    use squire_tests::{spoof_account, spoof_data, spoof_fluid_pairings};

    #[test]
    fn check_ins_function() {
        let (_, plyrs, rnds, _) = spoof_data(4);
        let mut sys = spoof_fluid_pairings();
        // You should be able to pair if no one has checked in
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        // Should should need at least N players to pair
        for id in plyrs.players.keys() {
            assert!(!sys.ready_to_pair(&plyrs, &rnds));
            sys.ready_player(*id);
        }
        // There are exactly N players, we should be able to attempt pairings
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        // Unready-ing a player should make the system unable to attempt a pairings
        for id in plyrs.players.keys() {
            sys.unready_player(*id);
            assert!(!sys.ready_to_pair(&plyrs, &rnds));
        }
    }

    #[test]
    fn simple_pair_all() {
        let (_, mut plyrs, rnds, _) = spoof_data(4);
        let mut sys = spoof_fluid_pairings();
        let standings = Standings::<StandardScore>::new(Vec::new());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        // Pairings should exist
        let pairings = sys.pair(&plyrs, &rnds, standings.clone()).unwrap();
        sys.update(&pairings);
        // There should be exactly one pairings (with 4 players) and no one else
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0].len(), 4);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        assert!(sys.pair(&plyrs, &rnds, standings.clone()).is_none());
        // Adding a 5th player
        let _ = plyrs.register_player(spoof_account());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        // Pairings should exist
        let pairings = sys.pair(&plyrs, &rnds, standings).unwrap();
        sys.update(&pairings);
        // There should be exactly one pairings (with 4 players) and no one else
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0].len(), 4);
        assert_eq!(pairings.rejected.len(), 0);
        // There should be one player in the queue, so three more players should make this ready to
        // pair
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        let id = plyrs.register_player(spoof_account()).unwrap();
        sys.ready_player(id);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        let id = plyrs.register_player(spoof_account()).unwrap();
        sys.ready_player(id);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        let id = plyrs.register_player(spoof_account()).unwrap();
        sys.ready_player(id);
        println!("{sys:?}");
        assert!(sys.ready_to_pair(&plyrs, &rnds));
    }

    #[test]
    fn top_of_queue_paired_first() {
        // If a player isn't paired, they should be the first one that is paired the next time
        let (_, plyrs, rnds, _) = spoof_data(5);
        let mut sys = spoof_fluid_pairings();
        let standings = Standings::<StandardScore>::new(Vec::new());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds, standings.clone()).unwrap();
        sys.update(&pairings);
        let in_queue = plyrs
            .players
            .keys()
            .find(|p| !pairings.paired[0].iter().any(|id| id == *p))
            .unwrap();
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds, standings).unwrap();
        sys.update(&pairings);
        assert_eq!(pairings.paired[0][0], *in_queue);
    }

    #[test]
    fn no_double_queued() {
        // If a player checking in should not cause them to count twice
        let (_, plyrs, rnds, _) = spoof_data(3);
        let mut sys = spoof_fluid_pairings();
        let standings = Standings::<StandardScore>::new(Vec::new());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(!sys.ready_to_pair(&plyrs, &rnds));

        // If a player checking in while in the queue should not cause them to count twice
        let (_, plyrs, rnds, _) = spoof_data(5);
        let mut sys = spoof_fluid_pairings();
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds, standings).unwrap();
        sys.update(&pairings);
        let in_queue = plyrs
            .players
            .keys()
            .find(|p| !pairings.paired[0].iter().any(|id| id == *p))
            .unwrap();
        sys.ready_player(pairings.paired[0][0]);
        sys.ready_player(pairings.paired[0][1]);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
        sys.ready_player(*in_queue);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
    }

    #[test]
    fn failed_to_re_pair() {
        // If a pairing is attempted and no pairings are found, everyone should be queued and no
        // one should be paired
        let (_, plyrs, mut rnds, _) = spoof_data(4);
        let mut sys = spoof_fluid_pairings();
        let standings = Standings::<StandardScore>::new(Vec::new());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds, standings.clone()).unwrap();
        let id = rnds.create_round(
            Utc::now(),
            pairings.paired[0].clone(),
            RoundContext::Contextless,
            0,
        );
        assert_eq!(rnds.opponents.len(), 4);
        println!("{:?}", rnds.get_round(&id).unwrap());
        // Everyone is paired, so there should be no round
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        let pairings = sys.pair(&plyrs, &rnds, standings.clone()).unwrap();
        sys.update(&pairings);
        // There should be no pairings since we aren't repairing people
        assert_eq!(pairings.paired.len(), 0);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));

        // TODO: Provide description
        let (_, plyrs, mut rnds, _) = spoof_data(6);
        let mut sys = spoof_fluid_pairings();
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds, standings.clone()).unwrap();
        sys.update(&pairings);
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.rejected.len(), 0);
        let _id = rnds.create_round(
            Utc::now(),
            pairings.paired[0].clone(),
            RoundContext::Contextless,
            0,
        );
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        let pairings = sys.pair(&plyrs, &rnds, standings).unwrap();
        sys.update(&pairings);
        assert_eq!(pairings.paired.len(), 0);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
    }

    #[test]
    fn remove_queued_player() {
        // If a player checking in while in the queue should not cause them to count twice
        let (_, plyrs, rnds, _) = spoof_data(5);
        let mut sys = spoof_fluid_pairings();
        let standings = Standings::<StandardScore>::new(Vec::new());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds, standings).unwrap();
        sys.update(&pairings);
        let in_queue = plyrs
            .players
            .keys()
            .find(|p| !pairings.paired[0].iter().any(|id| id == *p))
            .unwrap();
        sys.ready_player(pairings.paired[0][0]);
        sys.ready_player(pairings.paired[0][1]);
        sys.ready_player(pairings.paired[0][2]);
        println!("{sys:?}");
        assert!(sys.ready_to_pair(&plyrs, &rnds));
        sys.unready_player(*in_queue);
        assert!(!sys.ready_to_pair(&plyrs, &rnds));
    }
}
