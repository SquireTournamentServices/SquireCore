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
        let returnhtml = self
            .id
            .map(|id| {
                tourn
                    .get_round(&id.into())
                    .map(|rnd| {
                        html! {
                            <>
                            <h4>{ format!("Round #{} at table #{}", rnd.match_number, rnd.table_number) }</h4>
                            <p>{ format!("# of players : {}", rnd.players.len()) }</p>
                            <p>{ format!("Active : {}", rnd.is_active()) }</p>
                            <p>{ format!("Bye : {}", rnd.is_bye()) }</p>
                            </>
                        }
                    })
                    .unwrap_or_else(|_| html!{
                        <h4>{"Round not found"}</h4>
                    })
            })
            .unwrap_or_else(|| html!{
                <h4>{"No round selected"}</h4>
            });
        return html!{
            <div class="m-2">{returnhtml}</div>
        };
    }
}
