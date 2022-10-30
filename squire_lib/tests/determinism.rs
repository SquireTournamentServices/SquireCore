#[cfg(test)]
mod tests {
    use chrono::Utc;
    use squire_lib::{accounts::SquireAccount, tournament::TournamentPreset, operations::{TournOp, JudgeOp, AdminOp}, identifiers::AdminId, settings::{TournamentSetting, PairingSetting}};

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
        let op = TournOp::AdminOp(a_id, AdminOp::UpdateTournSetting(TournamentSetting::PairingSetting(PairingSetting::MatchSize(2))));
        tourn_one.apply_op(now, op.clone()).unwrap().assume_nothing();
        tourn_two.apply_op(now, op).unwrap().assume_nothing();
        tourn_one.id = tourn_two.id;
        assert_eq!(tourn_one, tourn_two);
        // Register the first player
        println!("Registering player one");
        let now = Utc::now();
        let op = TournOp::JudgeOp(a_id.into(), JudgeOp::RegisterGuest("PlayerOne".into()));
        let p_id_one = tourn_one.apply_op(now, op.clone()).unwrap().assume_register_player();
        let p_id_two = tourn_two.apply_op(now, op).unwrap().assume_register_player();
        assert_eq!(p_id_one, p_id_two);
        assert_eq!(tourn_one, tourn_two);
        // Register the second player
        println!("Registering player two");
        let now = Utc::now();
        let op = TournOp::JudgeOp(a_id.into(), JudgeOp::RegisterGuest("PlayerTwo".into()));
        let p_id_one = tourn_one.apply_op(now, op.clone()).unwrap().assume_register_player();
        let p_id_two = tourn_two.apply_op(now, op).unwrap().assume_register_player();
        assert_eq!(p_id_one, p_id_two);
        assert_eq!(tourn_one, tourn_two);
        // Start tournament
        println!("Starting tournament");
        let now = Utc::now();
        let op = TournOp::AdminOp(a_id, AdminOp::Start);
        tourn_one.apply_op(now, op.clone()).unwrap().assume_nothing();
        tourn_two.apply_op(now, op).unwrap().assume_nothing();
        // Pair the first round
        println!("Pairing first round");
        let now = Utc::now();
        let op = TournOp::AdminOp(a_id, AdminOp::PairRound);
        let r_id_one = tourn_one.apply_op(now, op.clone()).unwrap().assume_pair();
        let r_id_two = tourn_two.apply_op(now, op).unwrap().assume_pair();
        assert_eq!(r_id_one, r_id_two);
        assert_eq!(tourn_one, tourn_two);
    }
}
