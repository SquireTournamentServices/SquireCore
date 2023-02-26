#[cfg(test)]
mod tests {
    use chrono::Utc;

    use squire_tests::{get_fluid_seed, get_seed, spoof_account};

    use squire_lib::{
        error::TournamentError,
        identifiers::AdminId,
        operations::{AdminOp::*, TournOp},
        settings::*,
    };

    #[test]
    fn basic_tournament_settings() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.id.0.into();
        let mut tourn = admin.create_tournament(get_seed());
        // Basic tournament deck count bounds checking
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::MinDeckCount(5))
                )
            )
            .is_err());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::MinDeckCount(2))
                )
            )
            .is_ok());
        assert_eq!(2, tourn.min_deck_count);
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::MaxDeckCount(1))
                )
            )
            .is_err());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::MaxDeckCount(42))
                )
            )
            .is_ok());
        assert_eq!(42, tourn.max_deck_count);
    }

    #[test]
    fn check_pairings_guard() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.id.0.into();
        let mut tourn = admin.create_tournament(get_seed());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::PairingSetting(
                        PairingSetting::MatchSize(10)
                    ))
                )
            )
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(SwissPairingsSetting::DoCheckIns(true).into())
                )
            )
            .is_ok());
        let mut tourn = admin.create_tournament(get_fluid_seed());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::PairingSetting(
                        PairingSetting::MatchSize(10)
                    ))
                )
            )
            .is_ok());
        assert_eq!(
            Err(TournamentError::IncompatiblePairingSystem),
            tourn.apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(SwissPairingsSetting::DoCheckIns(true).into())
                )
            )
        );
    }
}
