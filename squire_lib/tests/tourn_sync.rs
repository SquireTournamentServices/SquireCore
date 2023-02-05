mod utils;

#[cfg(test)]
mod tests {
    use squire_lib::{
        accounts::SquireAccount,
        operations::{SyncStatus, TournOp},
        tournament::TournamentPreset,
        tournament_manager::TournamentManager,
    };

    use crate::utils::spoof_account;

    fn create_managers(
        creator: SquireAccount,
    ) -> (TournamentManager, TournamentManager, TournamentManager) {
        let tourn = TournamentManager::new(
            creator,
            "Test".to_owned(),
            TournamentPreset::Swiss,
            "Test".to_owned(),
        );

        (tourn.clone(), tourn.clone(), tourn)
    }

    #[test]
    fn simple_sync() {
        let creator = spoof_account();
        let (mut main, mut left, mut right) = create_managers(creator);
        let player_acc = spoof_account();
        left.apply_op(TournOp::RegisterPlayer(player_acc)).unwrap();
        let sync_request = left.sync_request();
        assert_eq!(sync_request.len(), 2);
        let sync = main.attempt_sync(sync_request.clone()).assume_completed();
        assert_eq!(sync.len(), 2);
        assert_eq!(sync, sync_request);
        assert_eq!(main.get_op_count(), 2);
        left.attempt_sync(sync).assume_completed();
        assert_eq!(left.tourn(), main.tourn());
        assert_eq!(left, main);
        assert_eq!(main.get_op_count(), 2);
        let sync_request = right.sync_request();
        assert_eq!(sync_request.len(), 1);
        let sync = main.attempt_sync(sync_request).assume_completed();
        assert_eq!(sync.len(), 2);
        right.attempt_sync(sync).assume_completed();
        assert_eq!(right.get_op_count(), 2);
        assert_eq!(right, main);
        assert_eq!(right, left);
    }
}
