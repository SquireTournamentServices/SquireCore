use squire_sdk::model::{
    rounds::{Round, RoundId},
    tournament::Tournament,
};
use yew::prelude::*;

pub struct SelectedRound {
    id: Option<RoundId>,
}

pub struct SelectedRoundViewResult {
    found: bool,
    match_number: u64,
    table_number: u64,
}
impl Default for SelectedRoundViewResult {
    fn default() -> Self {
        SelectedRoundViewResult {
            found : false,
            match_number : 0,
            table_number : 0
        }
    }
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
        let result: SelectedRoundViewResult = self
            .id
            .map(|id| {
                tourn
                    .get_round(&id.into())
                    .map(|rnd| {
                        SelectedRoundViewResult {
                            found : true,
                            match_number : rnd.match_number,
                            table_number : rnd.table_number
                        }
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        html! {
            <div class="my-1 mx-3">
                <h4>{ format!("Round #{} at Table #{}", result.match_number, result.table_number) }</h4>
            </div>
        }
    }
}
