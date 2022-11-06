#[cfg(test)]
mod tests {
    use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher}};

    use chrono::Utc;
    use deterministic_hash::DeterministicHasher;
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
    fn basic_tournament_settings() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.user_id.0.into();
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
        assert_eq!(2, tourn.get_state().min_deck_count);
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
        assert_eq!(42, tourn.get_state().max_deck_count);
    }

    #[test]
    fn check_pairings_guard() {
        let admin = spoof_account();
        let admin_id: AdminId = admin.user_id.0.into();
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
