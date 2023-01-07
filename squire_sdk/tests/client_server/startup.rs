use super::get_client;

#[tokio::test]
async fn startup() {
    let _ = get_client().await;
}

/*
#[tokio::test]
async fn lots_of_requests() {
    send_lots_of_requests(0).await
}

async fn send_lots_of_requests(i: usize) {
    let mut client = get_client().await;
    println!("Getting client #{i}");
    for i in 0..1000 {
        println!("Sending request #{i}");
        let resp = client.verify().await;
        println!("Resp: {resp:?}");
        resp.unwrap();
    }
}

#[tokio::test]
async fn lots_of_concurrent_requests() {
    let handles: Vec<_> = (0..10)
        .map(|i| tokio::spawn(async move { send_lots_of_requests(i).await }))
        .collect();
    for h in handles {
        h.await.unwrap();
    }
}
*/
