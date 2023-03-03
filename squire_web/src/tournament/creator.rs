use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use squire_sdk::tournaments::TournamentId;

use crate::{Route, CLIENT};

#[function_component(TournamentCreator)]
pub fn tournament_creator() -> Html {
    let navigator = use_navigator().unwrap();

    let onclick_callback = Callback::from(move |_| {
        let client = CLIENT.get().unwrap();
        let id = client.state.create_tournament();
        navigator.push(&Route::Tourn { id })
    });
    html! {
        <div>
            <button onclick={onclick_callback}>{ "Create tournament!" }</button>
        </div>
    }
}
