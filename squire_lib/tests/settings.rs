#[cfg(test)]
mod tests {
    use chrono::Utc;
    use squire_lib::{
        error::TournamentError,
        identifiers::AdminId,
        operations::{AdminOp::*, TournOp},
        settings::*,
    };
    use squire_tests::{get_fluid_seed, get_seed, spoof_account};

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
                    UpdateTournSetting(TournamentSetting::GeneralSetting(
                        GeneralSetting::MinDeckCount(5)
                    ))
                )
            )
            .is_err());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::GeneralSetting(
                        GeneralSetting::MinDeckCount(2)
                    ))
                )
            )
            .is_err());
        assert_eq!(0, tourn.settings.min_deck_count);
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::GeneralSetting(
                        GeneralSetting::MaxDeckCount(1)
                    ))
                )
            )
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::GeneralSetting(
                        GeneralSetting::MaxDeckCount(42)
                    ))
                )
            )
            .is_ok());
        assert_eq!(42, tourn.settings.max_deck_count);
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::GeneralSetting(
                        GeneralSetting::MinDeckCount(40)
                    ))
                )
            )
            .is_ok());
        assert_eq!(40, tourn.settings.min_deck_count);
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
                    UpdateTournSetting(TournamentSetting::PairingSetting(PairingSetting::Common(
                        CommonPairingSetting::MatchSize(10)
                    )))
                )
            )
            .is_ok());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(SwissPairingSetting::DoCheckIns(true).into())
                )
            )
            .is_ok());
        let mut tourn = admin.create_tournament(get_fluid_seed());
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(TournamentSetting::PairingSetting(PairingSetting::Common(
                        CommonPairingSetting::MatchSize(10)
                    )))
                )
            )
            .is_ok());
        assert_eq!(
            Err(TournamentError::IncompatiblePairingSystem),
            tourn.apply_op(
                Utc::now(),
                TournOp::AdminOp(
                    admin_id,
                    UpdateTournSetting(SwissPairingSetting::DoCheckIns(true).into())
                )
            )
        );
    }
}
