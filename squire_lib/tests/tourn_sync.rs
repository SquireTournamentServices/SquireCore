mod utils;

#[cfg(test)]
mod tests {
    use squire_lib::{
        accounts::SquireAccount, operations::{TournOp, SyncStatus}, tournament::TournamentPreset,
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
        left.apply_op(TournOp::RegisterPlayer(player_acc.clone()))
            .unwrap();
        let sync_request = left.sync_request();
        assert_eq!(sync_request.len(), 2);
        let sync = main.attempt_sync(sync_request.clone()).assume_completed();
        assert_eq!(sync.len(), 2);
        assert_eq!(sync, sync_request);
        todo!()
    }
}
