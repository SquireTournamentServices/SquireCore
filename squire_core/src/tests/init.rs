use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::OnceCell;
use rocket::local::asynchronous::Client;

static SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static SERVER: OnceCell<Client> = OnceCell::new();


async fn init() {
    let client = Client::tracked(crate::init())
        .await
        .expect("Could not initialize server");
    SERVER
        .set(client)
        .expect("Could not set initialized server")
}

pub(crate) async fn get_server() -> &'static Client {
    if let Ok(_) =
        SERVER_STARTED.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
    {
        init().await;
    }
    SERVER.wait()
}
