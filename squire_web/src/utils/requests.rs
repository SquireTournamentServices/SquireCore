use gloo_net::http::Request;
use squire_sdk::{
    api::GET_TOURNAMENT_ROUTE, tournaments::TournamentId,
};

use crate::CLIENT;

pub async fn fetch_tournament(id: TournamentId) -> bool {
    web_sys::console::log_1(&format!("Fetching tournament id: {id}").into());
    if let Ok(resp) = Request::get(&GET_TOURNAMENT_ROUTE.replace(&[id.to_string().as_str()]))
        .send()
        .await
    {
        web_sys::console::log_1(&format!("Decoding fetched data for: {id}").into());
        if let Ok(tourn) = resp.json().await {
            web_sys::console::log_1(&format!("Importing tournament id: {id}").into());
            CLIENT.get().unwrap().import_tourn(tourn).process().await;
            return true;
        }
    }
    false
}
