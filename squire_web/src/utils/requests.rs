use gloo_net::http::Request;
use squire_sdk::{
    api::GET_TOURNAMENT_ROUTE, client::state::ClientState, tournaments::TournamentId,
};

use crate::CLIENT;

pub async fn fetch_tournament(id: TournamentId) -> bool {
    web_sys::console::log_1(&format!("Importing tournament id: {id}").into());
    if let Ok(resp) = Request::get(&GET_TOURNAMENT_ROUTE.replace(&[id.to_string().as_str()]))
        .send()
        .await
    {
        if let Ok(tourn) = resp.json().await {
            CLIENT.get().unwrap().state.import_tournament(tourn);
            return true;
        }
    }
    false
}
