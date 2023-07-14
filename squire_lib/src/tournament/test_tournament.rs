#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        accounts::{SharingPermissions, SquireAccount},
        admin::Admin,
        error::TournamentError,
        operations::{AdminOp, PlayerOp, TournOp},
        rounds::RoundResult,
    };
    use crate::tournament::tournament::Tournament;
    use crate::tournament::tournament_preset::TournamentPreset;
    use crate::tournament_seed::TournamentSeed;

    fn spoof_account() -> SquireAccount {
        let id = Uuid::new_v4().into();
        SquireAccount {
            id,
            user_name: id.to_string(),
            display_name: id.to_string(),
            gamer_tags: HashMap::new(),
            permissions: SharingPermissions::Everything,
        }
    }

    #[test]
    fn players_in_paired_rounds() {
        let mut tourn =
            Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Test".into());
        assert_eq!(tourn.pairing_sys.common.match_size, 2);
        let acc = spoof_account();
        let admin = Admin::new(acc);
        tourn.admins.insert(admin.id, admin.clone());
        let acc = spoof_account();
        tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(acc, None))
            .unwrap()
            .assume_register_player();
        let acc = spoof_account();
        tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(acc, None))
            .unwrap()
            .assume_register_player();
        tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin.id, AdminOp::Start))
            .unwrap()
            .assume_nothing();
        let pairings = tourn.create_pairings().unwrap();
        let r_ids = tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::PairRound(pairings)),
            )
            .unwrap()
            .assume_pair();
        assert_eq!(r_ids.len(), 1);
        let rnd = tourn.get_round_by_id(&r_ids[0]).unwrap();
        assert_eq!(rnd.players.len(), 2);
    }

    #[test]
    fn confirm_all_rounds_test() {
        let mut tourn =
            Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Test".into());
        assert_eq!(tourn.pairing_sys.common.match_size, 2);
        let acc = spoof_account();
        let admin = Admin::new(acc);
        tourn.admins.insert(admin.id, admin.clone());
        let mut plyrs = Vec::with_capacity(4);
        for _ in 0..4 {
            let acc = spoof_account();
            let id = tourn
                .apply_op(Utc::now(), TournOp::RegisterPlayer(acc, None))
                .unwrap()
                .assume_register_player();
            plyrs.push(id);
        }
        tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin.id, AdminOp::Start))
            .unwrap()
            .assume_nothing();
        // Pair the first round
        let pairings = tourn.create_pairings().unwrap();
        let rnds = tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::PairRound(pairings)),
            )
            .unwrap()
            .assume_pair();
        assert_eq!(rnds.len(), 2);
        let r_id = tourn
            .round_reg
            .get_player_active_round(&plyrs[0])
            .unwrap()
            .id;
        tourn
            .apply_op(
                Utc::now(),
                TournOp::PlayerOp(
                    plyrs[0],
                    PlayerOp::RecordResult(r_id, RoundResult::Wins(plyrs[0], 1)),
                ),
            )
            .unwrap()
            .assume_nothing();
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::ConfirmAllRounds),
            )
            .is_err());
        for p in plyrs {
            let r_id = tourn.round_reg.get_player_active_round(&p).unwrap().id;
            tourn
                .apply_op(
                    Utc::now(),
                    TournOp::PlayerOp(p, PlayerOp::RecordResult(r_id, RoundResult::Wins(p, 1))),
                )
                .unwrap()
                .assume_nothing();
        }
        tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::ConfirmAllRounds),
            )
            .unwrap()
            .assume_nothing();
        // Pair the second round
        let pairings = tourn.create_pairings().unwrap();
        tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::PairRound(pairings)),
            )
            .unwrap()
            .assume_pair();
    }

    #[test]
    fn valid_tournament_names() {
        fn seed(name: &str) -> Result<TournamentSeed, TournamentError> {
            TournamentSeed::new(
                name.to_string(),
                TournamentPreset::Fluid,
                "Test".to_string(),
            )
        }

        assert!(seed("").is_err());
        assert!(seed("😄").is_ok());
        assert!(seed("abc").is_ok());
        assert!(seed("_!(:)@").is_ok());
        assert!(seed("Magic: the Gathering").is_ok());
        assert!(seed("Prophecy: the Body").is_ok());
    }
}
