#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        str::FromStr,
    };

    use chrono::{DateTime, Utc};
    use deterministic_hash::DeterministicHasher;
    use uuid::Uuid;
    
    use squire_lib::{
        accounts::SquireAccount,
        identifiers::AdminId,
        operations::{AdminOp, JudgeOp, TournOp},
        settings::{PairingSetting, TournamentSetting},
        tournament::TournamentPreset,
    };

    // NOTE: While we can't test different architectures on the same machine, this is controlled
    // for in CI
    #[test]
    fn consistant_hashing() {
        let mut now_and_id_hasher = DeterministicHasher::new(DefaultHasher::new());
        let mut now_and_vec_hasher = DefaultHasher::new();
        // Hash a timestamp
        let mut hasher = DefaultHasher::new();
        let now: DateTime<Utc> = DateTime::from_str("2022-11-06 01:10:43.945027291 UTC").unwrap();
        now.hash(&mut hasher);
        now.hash(&mut now_and_id_hasher);
        now.hash(&mut now_and_vec_hasher);
        let hash = hasher.finish();
        assert_eq!(hash, 6743072779167678941);
        // Hash a single id
        let mut hasher = DefaultHasher::new();
        let id = Uuid::from_str("14a0544b-d4a4-4056-a1c0-6905db264ecf").unwrap();
        id.hash(&mut hasher);
        id.hash(&mut now_and_id_hasher);
        let hash = hasher.finish();
        let now_and_id_hash = hasher.finish();
        assert_eq!(hash, 12099548975201988587);
        assert_eq!(now_and_id_hash, 12099548975201988587);
        // Hash a vec of ids
        let mut hasher = DefaultHasher::new();
        let ids = vec![
            Uuid::from_str("ea476c0c-8d58-4429-8a9e-5e719c000d7a").unwrap(),
            Uuid::from_str("6d306232-44a0-4bf1-b971-d8d77bccb754").unwrap(),
            Uuid::from_str("7a00ca44-7ab8-4b93-a068-29bbf7e1a3fb").unwrap(),
            Uuid::from_str("22103c9e-58c3-4143-9e70-bff4f531ad89").unwrap(),
        ];
        ids.hash(&mut hasher);
        ids.hash(&mut now_and_vec_hasher);
        let hash = hasher.finish();
        let now_and_vec_hash = hasher.finish();
        assert_eq!(hash, 10483062192994910974);
        assert_eq!(now_and_vec_hash, 10483062192994910974);
    }

    #[test]
    fn basic_determinism() {
        let account = SquireAccount::new("Test".into(), "Test".into());
        let a_id: AdminId = account.user_id.0.into();
        let mut tourn_one = account
            .create_tournament("Test".into(), TournamentPreset::Swiss, "Pioneer".into())
            .extract();
        let mut tourn_two = account
            .create_tournament("Test".into(), TournamentPreset::Swiss, "Pioneer".into())
            .extract();
        let now = Utc::now();
        let op = TournOp::AdminOp(
            a_id,
            AdminOp::UpdateTournSetting(TournamentSetting::PairingSetting(
                PairingSetting::MatchSize(2),
            )),
        );
        tourn_one
            .apply_op(now, op.clone())
            .unwrap()
            .assume_nothing();
        tourn_two.apply_op(now, op).unwrap().assume_nothing();
        tourn_one.id = tourn_two.id;
        assert_eq!(tourn_one, tourn_two);
        // Register the first player
        println!("Registering player one");
        let now = Utc::now();
        let op = TournOp::JudgeOp(a_id.into(), JudgeOp::RegisterGuest("PlayerOne".into()));
        let p_id_one = tourn_one
            .apply_op(now, op.clone())
            .unwrap()
            .assume_register_player();
        let p_id_two = tourn_two
            .apply_op(now, op)
            .unwrap()
            .assume_register_player();
        assert_eq!(p_id_one, p_id_two);
        assert_eq!(tourn_one, tourn_two);
        // Register the second player
        println!("Registering player two");
        let now = Utc::now();
        let op = TournOp::JudgeOp(a_id.into(), JudgeOp::RegisterGuest("PlayerTwo".into()));
        let p_id_one = tourn_one
            .apply_op(now, op.clone())
            .unwrap()
            .assume_register_player();
        let p_id_two = tourn_two
            .apply_op(now, op)
            .unwrap()
            .assume_register_player();
        assert_eq!(p_id_one, p_id_two);
        assert_eq!(tourn_one, tourn_two);
        // Start tournament
        println!("Starting tournament");
        let now = Utc::now();
        let op = TournOp::AdminOp(a_id, AdminOp::Start);
        tourn_one
            .apply_op(now, op.clone())
            .unwrap()
            .assume_nothing();
        tourn_two.apply_op(now, op).unwrap().assume_nothing();
        // Pair the first round
        println!("Creating first pairings");
        let now = Utc::now();
        let op = TournOp::AdminOp(a_id, AdminOp::CreatePairings);
        let pairings = tourn_one
            .apply_op(now, op.clone())
            .unwrap()
            .assume_create_pairings();
        let _ = tourn_two
            .apply_op(now, op)
            .unwrap()
            .assume_create_pairings();
        assert_eq!(tourn_one, tourn_two);
        println!("Creating rounds");
        let now = Utc::now();
        let op = TournOp::AdminOp(a_id, AdminOp::PairRound(pairings));
        let rnds_one = tourn_one.apply_op(now, op.clone()).unwrap().assume_pair();
        let rnds_two = tourn_two.apply_op(now, op).unwrap().assume_pair();
        assert_eq!(rnds_one, rnds_two);
        assert_eq!(tourn_one, tourn_two);
    }
}
