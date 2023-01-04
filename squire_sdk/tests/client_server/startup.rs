use super::get_client;

#[tokio::test]
async fn startup() {
    let _ = get_client().await;
}
