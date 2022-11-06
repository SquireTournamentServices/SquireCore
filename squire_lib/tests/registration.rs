#[cfg(test)]
mod tests {
    use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher}};

    use chrono::Utc;
    use deterministic_hash::DeterministicHasher;
    use squire_lib::{
        accounts::{SharingPermissions, SquireAccount},
        identifiers::AdminId,
        operations::{AdminOp::*, JudgeOp::*, TournOp},
        tournament::TournamentPreset,
    };
    use uuid::Uuid;

    fn spoof_account() -> SquireAccount {
        let now = Utc::now();
        let mut hasher = DeterministicHasher::new(DefaultHasher::new());
        now.hash(&mut hasher);
        let upper = hasher.finish();
        upper.hash(&mut hasher);
        let lower = hasher.finish();
        let id = Uuid::from_u64_pair(upper, lower);
        SquireAccount {
            user_name: id.to_string(),
            display_name: id.to_string(),
            gamer_tags: HashMap::new(),
            user_id: id.into(),
            permissions: SharingPermissions::Everything,
        }
    }

    #[test]
    fn regular_reg_tests() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.user_id.0.into();
        let mut tourn = admin.create_tournament(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        // Reg status is respected
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_ok());
        // Starting closes reg
        assert!(tourn.apply_op(TournOp::AdminOp(admin_id, Start)).is_ok());
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_ok());
        // Frozen tournament will never let people in
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn.apply_op(TournOp::AdminOp(admin_id, Freeze)).is_ok());
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_err());
        assert!(tourn.apply_op(TournOp::AdminOp(admin_id, Thaw)).is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_ok());
        assert!(tourn.apply_op(TournOp::AdminOp(admin_id, Freeze)).is_ok());
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_err());
        assert!(tourn.apply_op(TournOp::AdminOp(admin_id, Thaw)).is_ok());
        // Players can't join closed tournaments
        assert!(tourn.apply_op(TournOp::AdminOp(admin_id, End)).is_ok());
        assert!(tourn
            .apply_op(TournOp::RegisterPlayer(spoof_account()))
            .is_err());
    }

    #[test]
    fn admin_reg_tests() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.user_id.0.into();
        let mut tourn = admin.create_tournament(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        // Reg status is respected
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(false)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id, UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_ok());
        // Starting closes reg
        assert!(tourn.apply_op(TournOp::AdminOp(admin_id, Start)).is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_ok());
        // Frozen tournament will never let people in
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), UpdateReg(true)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), Freeze))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), UpdateReg(false)))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), Thaw))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), UpdateReg(false)))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), Freeze))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), Thaw))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_ok());
        // Players can't join closed tournaments
        assert!(tourn
            .apply_op(TournOp::AdminOp(admin_id.into(), End))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::JudgeOp(
                admin_id.into(),
                AdminRegisterPlayer(spoof_account())
            ))
            .is_err());
    }
}
