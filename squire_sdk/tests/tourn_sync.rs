mod utils;

#[cfg(test)]
mod tests {
    use squire_sdk::{
        model::{
            accounts::SquireAccount,
        },
        sync::TournamentManager,
    };

    use crate::utils::{get_seed};

    fn create_managers(
        creator: SquireAccount,
    ) -> (TournamentManager, TournamentManager, TournamentManager) {
        let tourn = TournamentManager::new(creator, get_seed());

        (tourn.clone(), tourn.clone(), tourn)
    }

    /* TODO: These tests ensure internal invariants of the syncing process, so they should not be
     * integration tests
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

    #[test]
    fn simple_player_id_replacement() {
        /* ---- Set up ---- */
        let creator = spoof_account();
        let admin_id: AdminId = (*creator.id).into();
        let (mut main, mut left, _) = create_managers(creator);

        /* ---- Register a player in the left tournament ---- */
        let op = TournOp::JudgeOp(admin_id.into(), JudgeOp::RegisterGuest("Tester".to_owned()));
        let left_p_id = left.apply_op(op.clone()).unwrap().assume_register_player();
        let sync_request = left.sync_request();
        assert_eq!(left.get_op_count(), 2);
        assert_eq!(sync_request.len(), 2);

        /* ---- Register the same player in the main tournament (without syncing) ---- */
        let main_p_id = main.apply_op(op).unwrap().assume_register_player();
        assert_eq!(main.get_op_count(), 2);

        /* ---- Apply a sync request to main ---- */
        let sync = main.attempt_sync(sync_request.clone()).assume_completed();
        assert_eq!(sync.len(), 2);
        assert_ne!(sync, sync_request);
        assert_eq!(main.get_op_count(), 2);

        /* ---- Ensure that player ids get updated correctly --- */
        left.overwrite(sync.into()).unwrap();
        assert!(main.get_player(&main_p_id.into()).is_ok());
        assert!(left.get_player(&main_p_id.into()).is_ok());
        assert!(main.get_player(&left_p_id.into()).is_err());
        assert!(left.get_player(&left_p_id.into()).is_err());
    }
    */
}
