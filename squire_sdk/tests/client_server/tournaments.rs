use squire_sdk::{
    client::{ClientError, ClientState},
    tournaments::TournamentPreset,
};

use crate::client_server::get_client;


#[tokio::test]
async fn create_tournament_requires_login() {
    let mut client = get_client().await;
    let resp = client
        .create_tournament(
            "Test".to_owned(),
            TournamentPreset::Swiss,
            "Test".to_owned(),
        )
        .await;

    match resp {
        Err(ClientError::NotLoggedIn) => {}
        Err(err) => {
            panic!("Tournament creation failed in an unexpected way: {err:?}");
        }
        Ok(_) => {
            panic!("Tournament creation successed when it should have failed!");
        }
    }
}

/* TODO: Add back in once FailedConnection bug is sorted out
#[tokio::test]
async fn create_tournament() {
    let mut client = get_client().await;
    let _ = client.verify().await.unwrap();
    assert!(client.is_verify());
    let id = client
        .create_tournament(
            "Test".to_owned(),
            TournamentPreset::Swiss,
            "Test".to_owned(),
        )
        .await
        .unwrap();
    client
        .state
        .query_tournament(&id, |t| assert_eq!(t.name.as_str(), "Test"))
        .unwrap();
}
*/
