use squire_sdk::{
    players::{Player, PlayerId},
    tournaments::Tournament, model::identifiers::{PlayerIdentifier, TypeId},
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
        let returnhtml = self
            .id
            .map(|id| {
                tourn
                    .get_player(&id.into())
                    .map(|plyr| {
                        html! {
                            <>
                            <h4>{ plyr.name.as_str() }</h4>
                            <p>{ format!("Gamertag : {}", plyr.game_name.clone().unwrap_or_else(|| "None".to_string())) }</p>
                            <p>{ format!("Can play : {}", plyr.can_play()) }</p>
                            <p>{ format!("Rounds : {}", tourn.get_player_rounds(&id.into()).unwrap_or_default().len() ) }</p>
                            <ul>
                            {
                                tourn.get_player_rounds(&id.into()).unwrap_or_default().into_iter()
                                    .map(|r| {
                                        html! {<li>{ format!("Match {} at table {}", r.match_number, r.table_number) }</li>}
                                    })
                                    .collect::<Html>()
                            }
                            </ul>
                            </>
                        }
                    })
                    .unwrap_or_else(|_| html!{
                        <h4>{"Player not found"}</h4>
                    })
            })
            .unwrap_or_else(|| html!{
                <h4>{"No player selected"}</h4>
            });
        return html!{
            <div class="m-2">{returnhtml}</div>
        };
    }
}
