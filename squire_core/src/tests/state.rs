#![cfg(feature = "db-tests")]

use async_session::chrono::Utc;
use squire_sdk::{
    server::state::ServerState,
    tournaments::{TournOp, TournamentManager, TournamentSummary},
};

use crate::state::{AppSettings, AppState};

async fn clear_database(settings: AppSettings) {
    AppState::new_with_settings(settings)
        .await
        .get_db()
        .drop(None)
        .await;
}

#[tokio::test]
async fn tournament_pages() {
    let settings = AppSettings::default().database_name("SquireTesting_tourn_pages");
    clear_database(settings.clone()).await;

    let state = AppState::new_with_settings(settings).await;
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
    let settings = AppSettings::default().database_name("SquireTesting_insert_fetch_tourn");
    clear_database(settings.clone()).await;

    let manager = TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());
    let state = AppState::new_with_settings(settings).await;

    state.persist_tourn(&manager).await;
    let retrieved_tourn = state
        .get_tourn(manager.id)
        .await
        .expect("Could not retrieve tournament from database");

    assert_eq!(manager, retrieved_tourn);
}

#[tokio::test]
async fn check_already_persisted() {
    let settings = AppSettings::default().database_name("SquireTesting_check_already_persisted");
    clear_database(settings.clone()).await;

    let mut manager =
        TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());
    let state = AppState::new_with_settings(settings).await;

    assert!(!state.persist_tourn(&manager).await);
    assert!(state.persist_tourn(&manager).await);

    let retrieved_tourn = state
        .get_tourn(manager.id)
        .await
        .expect("Could not retrieve tournament from database");
    assert_eq!(manager, retrieved_tourn);
}
