use yew::prelude::*;

use squire_sdk::{tournaments::Tournament, model::scoring::{ScoringSystem, StandardScoring}};

#[derive(Debug, Default)]
pub struct ScoringSettings {}

impl ScoringSettings {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        html! {
            <div>
                <h2>{ "Scoring Settings:" }</h2>
                <p>{ scoring_view(&tourn.scoring_sys) }</p>
            </div>
        }
    }
}

fn scoring_view(sys: &ScoringSystem) -> Html {
    match sys {
        ScoringSystem::Standard(standard) => standard_scoring_view(standard),
    }
}

fn standard_scoring_view(standard: &StandardScoring) -> Html {
    html! {
        <div>
            <h3>{ "Stanard Scoring Settings" }</h3>
            <p>{ format!("Match win points: {}", standard.match_win_points) }</p>
            <p>{ format!("Match draw points: {}", standard.match_draw_points) }</p>
            <p>{ format!("Match loss points: {}", standard.match_loss_points) }</p>
            <p>{ format!("Game win points: {}", standard.game_win_points) }</p>
            <p>{ format!("Game draw points: {}", standard.game_draw_points) }</p>
            <p>{ format!("Game loss points: {}", standard.game_loss_points) }</p>
            <p>{ format!("Bye points: {}", standard.bye_points) }</p>
            <p>{ format!("Include byes: {}", standard.include_byes) }</p>
            <p>{ format!("Include match points: {}", standard.include_match_points) }</p>
            <p>{ format!("Include game points: {}", standard.include_game_points) }</p>
            <p>{ format!("Include MWP: {}", standard.include_mwp) }</p>
            <p>{ format!("Include GWP: {}", standard.include_gwp) }</p>
            <p>{ format!("Include Opp MWP: {}", standard.include_opp_mwp) }</p>
            <p>{ format!("Include Opp GWP: {}", standard.include_opp_gwp) }</p>
        </div>
    }
}
