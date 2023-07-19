#![cfg(feature = "db-tests")]

use squire_sdk::{
    server::state::ServerState, sync::TournamentManager, tournaments::TournamentSummary,
};

use crate::state::{AppState, AppStateBuilder};

async fn clear_database(state: AppState) {
    state.get_db().drop(None).await;
}

#[tokio::test]
async fn tournament_pages() {
    let state = AppStateBuilder::new()
        .database_name("SquireTesting_tourn_pages")
        .build()
        .await;
    clear_database(state.clone()).await;

    let tournament_list = std::iter::repeat_with(|| {
        TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed())
    })
    .take(40)
    .collect::<Vec<_>>();

    // tournament listing should fetch from newest inserted to oldest inserting, so insert list in
    // reverse order
    for tournament in tournament_list.iter().rev() {
        state.persist_tourn(tournament).await;
    }

    // this way we can do direct comparisons
    assert_eq!(
        state.get_tourn_summaries(5..15).await,
        tournament_list[5..15]
            .iter()
            .map(TournamentSummary::from)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        state.get_tourn_summaries(0..10).await,
        tournament_list[0..10]
            .iter()
            .map(TournamentSummary::from)
            .collect::<Vec<_>>()
    );

    state
        .persist_tourn(&TournamentManager::new(
            squire_tests::spoof_account(),
            squire_tests::get_seed(),
        ))
        .await;

    assert_eq!(
        state.get_tourn_summaries(6..16).await,
        tournament_list[5..15]
            .iter()
            .map(TournamentSummary::from)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        state.get_tourn_summaries(1..11).await,
        tournament_list[0..10]
            .iter()
            .map(TournamentSummary::from)
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn insert_fetch_tourn() {
    let state = AppStateBuilder::new().database_name("SquireTesting_insert_fetch_tourn").build().await;
    clear_database(state.clone()).await;

    let manager = TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());

    state.persist_tourn(&manager).await;
    let retrieved_tourn = state
        .get_tourn(manager.id)
        .await
        .expect("Could not retrieve tournament from database");

    assert_eq!(manager, retrieved_tourn);
}

#[tokio::test]
async fn check_already_persisted() {
    let state = AppStateBuilder::new().database_name("SquireTesting_check_already_persisted").build().await;
    clear_database(state.clone()).await;

    let manager =
        TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());

    assert!(!state.persist_tourn(&manager).await);
    assert!(state.persist_tourn(&manager).await);

    let retrieved_tourn = state
        .get_tourn(manager.id)
        .await
        .expect("Could not retrieve tournament from database");
    assert_eq!(manager, retrieved_tourn);
}
