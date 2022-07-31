
#[cfg(test)]
mod tests {
    use squire_lib::{
        operations::TournOp::*,
        tournament::{Tournament, TournamentPreset}
    };
    use uuid::Uuid;

    #[test]
    fn regular_reg_tests() {
        let mut tourn = Tournament::from_preset(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        // Reg status is respected
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_ok());
        assert!(tourn.apply_op(UpdateReg(false)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(UpdateReg(true)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_ok());
        // Starting closes reg
        assert!(tourn.apply_op(Start()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(UpdateReg(true)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_ok());
        // Frozen tournament will never let people in
        assert!(tourn.apply_op(UpdateReg(true)).is_ok());
        assert!(tourn.apply_op(Freeze()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(UpdateReg(false)).is_err());
        assert!(tourn.apply_op(Thaw()).is_ok());
        assert!(tourn.apply_op(UpdateReg(false)).is_ok());
        assert!(tourn.apply_op(Freeze()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(Thaw()).is_ok());
        // Players can't join closed tournaments
        assert!(tourn.apply_op(End()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
    }

    #[test]
    fn admin_reg_tests() {
        let mut tourn = Tournament::from_preset(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        // Reg status is respected
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_ok());
        assert!(tourn.apply_op(UpdateReg(false)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(UpdateReg(true)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_ok());
        // Starting closes reg
        assert!(tourn.apply_op(Start()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(UpdateReg(true)).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_ok());
        // Frozen tournament will never let people in
        assert!(tourn.apply_op(UpdateReg(true)).is_ok());
        assert!(tourn.apply_op(Freeze()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(UpdateReg(false)).is_err());
        assert!(tourn.apply_op(Thaw()).is_ok());
        assert!(tourn.apply_op(UpdateReg(false)).is_ok());
        assert!(tourn.apply_op(Freeze()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
        assert!(tourn.apply_op(Thaw()).is_ok());
        // Players can't join closed tournaments
        assert!(tourn.apply_op(End()).is_ok());
        assert!(tourn.apply_op(RegisterPlayer(Uuid::new_v4().to_string())).is_err());
    }
}
