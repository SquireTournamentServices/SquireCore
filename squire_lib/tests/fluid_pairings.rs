#[cfg(test)]
mod tests {
    use std::time::Duration;

    use squire_lib::{
        accounts::{SharingPermissions, SquireAccount},
        fluid_pairings::FluidPairings,
        identifiers::UserAccountId,
        player_registry::PlayerRegistry,
        round_registry::RoundRegistry,
    };
    use uuid::Uuid;

    fn spoof_account() -> SquireAccount {
        let id: UserAccountId = Uuid::new_v4().into();
        SquireAccount {
            user_name: id.to_string(),
            display_name: id.to_string(),
            arena_name: None,
            mtgo_name: None,
            trice_name: None,
            user_id: id,
            do_share: SharingPermissions::Everything,
        }
    }

    fn spoof_data(count: usize) -> (FluidPairings, PlayerRegistry, RoundRegistry) {
        let mut plyrs = PlayerRegistry::new();
        for _ in 0..count {
            let _ = plyrs.add_player(spoof_account());
        }
        (
            FluidPairings::new(4),
            plyrs,
            RoundRegistry::new(0, Duration::from_secs(0)),
        )
    }

    #[test]
    fn check_ins_function() {
        let (mut sys, plyrs, _) = spoof_data(4);
        // You should be able to pair if no one has checked in
        assert!(!sys.ready_to_pair());
        // Should should need at least N players to pair
        for id in plyrs.players.keys() {
            assert!(!sys.ready_to_pair());
            sys.ready_player(*id);
        }
        // There are exactly N players, we should be able to attempt pairings
        assert!(sys.ready_to_pair());
        // Unready-ing a player should make the system unable to attempt a pairing
        for id in plyrs.players.keys() {
            sys.unready_player(*id);
            assert!(!sys.ready_to_pair());
        }
    }

    #[test]
    fn simple_pair_all() {
        let (mut sys, mut plyrs, rnds) = spoof_data(4);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        // Pairings should exist
        let pairings = sys.pair(&plyrs, &rnds).unwrap();
        // There should be exactly one pairing (with 4 players) and no one else
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0].len(), 4);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(!sys.ready_to_pair());
        // Adding a 5th player
        let _ = plyrs.add_player(spoof_account());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        // Pairings should exist
        let pairings = sys.pair(&plyrs, &rnds).unwrap();
        // There should be exactly one pairing (with 4 players) and no one else
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0].len(), 4);
        assert_eq!(pairings.rejected.len(), 0);
        // There should be one player in the queue, so three more players should make this ready to
        // pair
        assert!(!sys.ready_to_pair());
        let id = plyrs.add_player(spoof_account()).unwrap();
        sys.ready_player(id);
        assert!(!sys.ready_to_pair());
        let id = plyrs.add_player(spoof_account()).unwrap();
        sys.ready_player(id);
        assert!(!sys.ready_to_pair());
        let id = plyrs.add_player(spoof_account()).unwrap();
        sys.ready_player(id);
        println!("{:?}", sys);
        assert!(sys.ready_to_pair());
    }

    #[test]
    fn top_of_queue_paired_first() {
        // If a player isn't paired, they should be the first one that is paired the next time
        let (mut sys, plyrs, rnds) = spoof_data(5);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairing = sys.pair(&plyrs, &rnds).unwrap();
        let in_queue = plyrs
            .players
            .keys()
            .find(|p| pairing.paired[0].iter().find(|id| id == p).is_none()).unwrap();
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairing = sys.pair(&plyrs, &rnds).unwrap();
        assert_eq!(pairing.paired[0][0], *in_queue);
    }

    #[test]
    fn no_double_queued() {
        // If a player checking in should not cause them to count twice
        let (mut sys, plyrs, _) = spoof_data(3);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(!sys.ready_to_pair());
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(!sys.ready_to_pair());

        // If a player checking in while in the queue should not cause them to count twice
        let (mut sys, plyrs, rnds) = spoof_data(5);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairing = sys.pair(&plyrs, &rnds).unwrap();
        let in_queue = plyrs
            .players
            .keys()
            .find(|p| pairing.paired[0].iter().find(|id| id == p).is_none()).unwrap();
        sys.ready_player(pairing.paired[0][0]);
        sys.ready_player(pairing.paired[0][1]);
        assert!(!sys.ready_to_pair());
        sys.ready_player(*in_queue);
        assert!(!sys.ready_to_pair());
    }

    #[test]
    fn failed_to_re_pair() {
        // If a pairing in attempted and no pairings are found, everyone should be queued and no
        // one should be paired
        let (mut sys, plyrs, mut rnds) = spoof_data(4);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds).unwrap();
        let id = rnds.create_round();
        for p in &pairings.paired[0] {
            rnds.add_player_to_round(&id, *p).unwrap();
        }
        // Everyone is paired, so there should be no round
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(sys.ready_to_pair());
        let pairings = sys.pair(&plyrs, &rnds).unwrap();
        // There should be exactly one pairing (with 4 players) and no one else
        assert_eq!(pairings.paired.len(), 0);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(!sys.ready_to_pair());
        
        // TODO: Provide description
        let (mut sys, plyrs, mut rnds) = spoof_data(6);
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        let pairings = sys.pair(&plyrs, &rnds).unwrap();
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.rejected.len(), 0);
        let id = rnds.create_round();
        for p in &pairings.paired[0] {
            rnds.add_player_to_round(&id, *p).unwrap();
        }
        for id in plyrs.players.keys() {
            sys.ready_player(*id);
        }
        assert!(sys.ready_to_pair());
        let pairings = sys.pair(&plyrs, &rnds).unwrap();
        assert_eq!(pairings.paired.len(), 0);
        assert_eq!(pairings.rejected.len(), 0);
        assert!(!sys.ready_to_pair());
    }
}
