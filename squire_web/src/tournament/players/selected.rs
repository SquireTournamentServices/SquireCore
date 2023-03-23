use squire_sdk::{
    players::{Player, PlayerId},
    tournaments::Tournament, model::identifiers::{PlayerIdentifier, TypeId},
};
use yew::prelude::*;

pub struct SelectedPlayer {
    id: Option<PlayerId>,
}

pub struct SelectedPlayerViewResult {
    found: bool,
    name: String,
    gamertag: String,
}
impl Default for SelectedPlayerViewResult {
    fn default() -> Self {
        SelectedPlayerViewResult {
            found : false,
            name : "...".to_owned(),
            gamertag : "...".to_owned()
        }
    }
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
        let result: SelectedPlayerViewResult = self
            .id
            .map(|id| {
                tourn
                    .get_player(&id.into())
                    .map(|plyr| {
                        SelectedPlayerViewResult {
                            found : true,
                            name : plyr.name.to_string(),
                            gamertag : plyr.game_name.clone().unwrap_or("None".to_string())
                        }
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        html! {
            <div class="my-1 mx-3">
                <h4>{ result.name }</h4>
                <p>{ format!("Gamertag : {}", result.gamertag) }</p>
            </div>
        }
    }
}
