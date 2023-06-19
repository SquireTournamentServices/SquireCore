use squire_sdk::tournaments::{TournamentId, TournamentPreset};
use yew::{functional::*, prelude::*};
use yew_router::prelude::*;

use crate::{Route, CLIENT};

#[function_component(TournamentCreator)]
pub fn tournament_creator() -> Html {
    todo!()
    /*
    let navigator = use_navigator().unwrap();

    let onclick_callback = Callback::from(move |_| {
        let client = CLIENT.get().unwrap();
        let id = client.create_tournament(
            "Some Tournament".to_owned(),
            TournamentPreset::Swiss,
            "Some Tournament".to_owned(),
        ).await;
        navigator.push(&Route::Tourn { id })
    });
    html! {
        <div>
            <button onclick={onclick_callback}>{ "Create tournament!" }</button>
        </div>
    }
    */
}
