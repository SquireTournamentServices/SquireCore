#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use squire_lib::{
        accounts::{SharingPermissions, SquireAccount},
        error::TournamentError,
        identifiers::AdminId,
        operations::{AdminOp::*, TournOp},
        settings::*,
        tournament::TournamentPreset,
    };
    use uuid::Uuid;

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
    fn basic_tournament_settings() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.id.0.into();
        let mut tourn = admin.create_tournament(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        // Basic tournament deck count bounds checking
        assert!(tourn
            .apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(TournamentSetting::MinDeckCount(5))
            ))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(TournamentSetting::MinDeckCount(2))
            ))
            .is_ok());
        assert_eq!(2, tourn.tourn().min_deck_count);
        assert!(tourn
            .apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(TournamentSetting::MaxDeckCount(1))
            ))
            .is_err());
        assert!(tourn
            .apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(TournamentSetting::MaxDeckCount(42))
            ))
            .is_ok());
        assert_eq!(42, tourn.tourn().max_deck_count);
    }

    #[test]
    fn check_pairings_guard() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.id.0.into();
        let mut tourn = admin.create_tournament(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        assert!(tourn
            .apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(TournamentSetting::PairingSetting(
                    PairingSetting::MatchSize(10)
                ))
            ))
            .is_ok());
        assert!(tourn
            .apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(SwissPairingsSetting::DoCheckIns(true).into())
            ))
            .is_ok());
        let mut tourn = admin.create_tournament(
            "Test Tournament".into(),
            TournamentPreset::Fluid,
            "Pioneer".into(),
        );
        assert!(tourn
            .apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(TournamentSetting::PairingSetting(
                    PairingSetting::MatchSize(10)
                ))
            ))
            .is_ok());
        assert_eq!(
            Err(TournamentError::IncompatiblePairingSystem),
            tourn.apply_op(TournOp::AdminOp(
                admin_id,
                UpdateTournSetting(SwissPairingsSetting::DoCheckIns(true).into())
            ))
        );
    }
}
