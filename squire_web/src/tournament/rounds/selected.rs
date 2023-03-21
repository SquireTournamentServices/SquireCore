use squire_sdk::model::{
    rounds::{Round, RoundId},
    tournament::Tournament,
};
use yew::prelude::*;

pub struct SelectedRound {
    id: Option<RoundId>,
}

impl SelectedRound {
    pub fn new() -> Self {
        Self { id: None }
    }

    pub fn update(&mut self, id: Option<RoundId>) -> bool {
        let digest = self.id != id;
        self.id = id;
        digest
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        html! { <p>{ self
            .id
            .map(|id| {
                tourn
                    .get_round(&id.into())
                    .map(|rnd| format!("Round #{} at table #{}", rnd.match_number, rnd.table_number))
                    .unwrap_or_else(|_| "Round not found!!!".to_owned())
            })
            .unwrap_or_else(|| "No round selected!!".to_owned())
        }</p> }
    }
}
