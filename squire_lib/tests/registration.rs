#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use squire_lib::{
        accounts::{SharingPermissions, SquireAccount},
        identifiers::AdminId,
        operations::TournOp::*,
        tournament::TournamentPreset,
    };
    use uuid::Uuid;

    fn spoof_account() -> SquireAccount {
        let id = Uuid::new_v4();
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
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_ok());
        assert!(tourn.apply_op(UpdateReg(admin_id, false)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_err());
        assert!(tourn.apply_op(UpdateReg(admin_id, true)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_ok());
        // Starting closes reg
        assert!(tourn.apply_op(Start(admin_id,)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_err());
        assert!(tourn.apply_op(UpdateReg(admin_id, true)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_ok());
        // Frozen tournament will never let people in
        assert!(tourn.apply_op(UpdateReg(admin_id, true)).is_ok());
        assert!(tourn.apply_op(Freeze(admin_id,)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_err());
        assert!(tourn.apply_op(UpdateReg(admin_id, false)).is_err());
        assert!(tourn.apply_op(Thaw(admin_id,)).is_ok());
        assert!(tourn.apply_op(UpdateReg(admin_id, false)).is_ok());
        assert!(tourn.apply_op(Freeze(admin_id,)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_err());
        assert!(tourn.apply_op(Thaw(admin_id,)).is_ok());
        // Players can't join closed tournaments
        assert!(tourn.apply_op(End(admin_id,)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(spoof_account())).is_err());
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
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_ok());
        assert!(tourn.apply_op(UpdateReg(admin_id, false)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_ok());
        assert!(tourn.apply_op(UpdateReg(admin_id, true)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_ok());
        // Starting closes reg
        assert!(tourn.apply_op(Start(admin_id,)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_ok());
        assert!(tourn.apply_op(UpdateReg(admin_id, true)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_ok());
        // Frozen tournament will never let people in
        assert!(tourn.apply_op(UpdateReg(admin_id, true)).is_ok());
        assert!(tourn.apply_op(Freeze(admin_id,)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_err());
        assert!(tourn.apply_op(UpdateReg(admin_id, false)).is_err());
        assert!(tourn.apply_op(Thaw(admin_id,)).is_ok());
        assert!(tourn.apply_op(UpdateReg(admin_id, false)).is_ok());
        assert!(tourn.apply_op(Freeze(admin_id,)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_err());
        assert!(tourn.apply_op(Thaw(admin_id,)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_ok());
        // Players can't join closed tournaments
        assert!(tourn.apply_op(End(admin_id,)).is_ok());
        assert!(tourn
            .apply_op(AdminRegisterPlayer(admin_id.into(), spoof_account()))
            .is_err());
    }
}
