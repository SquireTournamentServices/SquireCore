#[cfg(test)]
mod tests {
    use squire_lib::{
        error::TournamentError,
        operations::TournOp::*,
        settings::*,
        tournament::{Tournament, TournamentPreset},
    };

    #[test]
    fn basic_tournament_settings() {
        let mut tourn = Tournament::from_preset(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        // Basic tournament deck count bounds checking
        assert!(tourn
            .apply_op(UpdateTournSetting(TournamentSetting::MinDeckCount(5)))
            .is_err());
        assert!(tourn
            .apply_op(UpdateTournSetting(TournamentSetting::MinDeckCount(2)))
            .is_ok());
        assert_eq!(2, tourn.min_deck_count);
        assert!(tourn
            .apply_op(UpdateTournSetting(TournamentSetting::MaxDeckCount(1)))
            .is_err());
        assert!(tourn
            .apply_op(UpdateTournSetting(TournamentSetting::MaxDeckCount(42)))
            .is_ok());
        assert_eq!(42, tourn.max_deck_count);
    }

    #[test]
    fn check_pairings_guard() {
        let mut tourn = Tournament::from_preset(
            "Test Tournament".into(),
            TournamentPreset::Swiss,
            "Pioneer".into(),
        );
        assert_eq!(
            Err(TournamentError::IncompatiblePairingSystem),
            tourn.apply_op(UpdateTournSetting(TournamentSetting::PairingSetting(
                PairingSetting::Fluid(FluidPairingsSetting::MatchSize(10))
            )))
        );
        assert!(tourn
            .apply_op(UpdateTournSetting(TournamentSetting::PairingSetting(
                PairingSetting::Swiss(SwissPairingsSetting::MatchSize(10))
            )))
            .is_ok());
        let mut tourn = Tournament::from_preset(
            "Test Tournament".into(),
            TournamentPreset::Fluid,
            "Pioneer".into(),
        );
        assert_eq!(
            Err(TournamentError::IncompatiblePairingSystem),
            tourn.apply_op(UpdateTournSetting(TournamentSetting::PairingSetting(
                PairingSetting::Swiss(SwissPairingsSetting::MatchSize(10))
            )))
        );
        assert!(tourn
            .apply_op(UpdateTournSetting(TournamentSetting::PairingSetting(
                PairingSetting::Fluid(FluidPairingsSetting::MatchSize(10))
            )))
            .is_ok());
    }
}
