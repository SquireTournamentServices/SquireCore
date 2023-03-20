use squire_sdk::{
    players::{Player, PlayerId},
    tournaments::Tournament,
};
use yew::prelude::*;

pub struct SelectedPlayer {
    id: Option<PlayerId>,
}

impl SelectedPlayer {
    pub fn new() -> Self {
        Self { id: None }
    }

    pub fn update(&mut self, id: Option<PlayerId>) -> bool {
        let digest = self.id != id;
        self.id = id;
        digest
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        let txt = self
            .id
            .map(|id| {
                tourn
                    .get_player(&id.into())
                    .map(|plyr| plyr.name.as_str())
                    .unwrap_or("Player not found!!!")
            })
            .unwrap_or("No player selected!!");
        html! { <p>{ txt }</p> }
    }
}
