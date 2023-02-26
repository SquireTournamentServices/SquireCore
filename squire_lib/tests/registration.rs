#[cfg(test)]
mod tests {
    use chrono::Utc;

    use squire_tests::{get_seed, spoof_account};

    use squire_lib::{
        identifiers::AdminId,
        operations::{AdminOp::*, JudgeOp::*, TournOp},
    };

    #[test]
    fn regular_reg_tests() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.id.0.into();
        let mut tourn = admin.create_tournament(get_seed());
        // Reg status is respected
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_ok());
        // Starting closes reg
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Start))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_ok());
        // Frozen tournament will never let people in
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Freeze))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Thaw))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Freeze))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Thaw))
            .is_ok());
        // Players can't join closed tournaments
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, End))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .is_err());
    }

    #[test]
    fn admin_reg_tests() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.id.0.into();
        let mut tourn = admin.create_tournament(get_seed());
        // Reg status is respected
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_ok());
        // Starting closes reg
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Start))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_ok());
        // Frozen tournament will never let people in
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Freeze))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Thaw))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_ok());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Freeze))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_err());
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, Thaw))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_ok());
        // Players can't join closed tournaments
        assert!(tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin_id, End))
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::JudgeOp(admin_id.into(), AdminRegisterPlayer(spoof_account()))
            )
            .is_err());
    }
}
