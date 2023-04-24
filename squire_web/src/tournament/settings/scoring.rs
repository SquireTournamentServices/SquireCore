use yew::prelude::*;

use squire_sdk::{
    model::{scoring::{ScoringSystem, StandardScoring, ScoringStyle}, settings::{StandardScoringSettingsTree, ScoringSettingsTree, ScoringStyleSetting, ScoringStyleSettingsTree}},
    tournaments::Tournament,
};

#[derive(Debug)]
pub struct ScoringSettings {
    current: ScoringSettingsTree,
    to_change: ScoringSettingsTree,
}

impl ScoringSettings {
    pub fn new(tree: ScoringSettingsTree) -> Self {
        Self {
            current: tree.clone(),
            to_change: tree,
        }
    }

    pub fn view(&self) -> Html {
        html! {
            <div>
                <h2>{ "Scoring Settings:" }</h2>
                <p>{ scoring_view(&self.current.style) }</p>
            </div>
        }
    }
}

fn scoring_view(style: &ScoringStyleSettingsTree) -> Html {
    match &style {
        ScoringStyleSettingsTree::Standard(style) => standard_scoring_view(style),
    }
}

fn standard_scoring_view(settings: &StandardScoringSettingsTree) -> Html {
    html! {
        <div>
            <h3>{ "Stanard Scoring Settings" }</h3>
            <p>{ format!("Match win points: {}", settings.match_win_points) }</p>
            <p>{ format!("Match draw points: {}", settings.match_draw_points) }</p>
            <p>{ format!("Match loss points: {}", settings.match_loss_points) }</p>
            <p>{ format!("Game win points: {}", settings.game_win_points) }</p>
            <p>{ format!("Game draw points: {}", settings.game_draw_points) }</p>
            <p>{ format!("Game loss points: {}", settings.game_loss_points) }</p>
            <p>{ format!("Bye points: {}", settings.bye_points) }</p>
            <p>{ format!("Include byes: {}", settings.include_byes) }</p>
            <p>{ format!("Include match points: {}", settings.include_match_points) }</p>
            <p>{ format!("Include game points: {}", settings.include_game_points) }</p>
            <p>{ format!("Include MWP: {}", settings.include_mwp) }</p>
            <p>{ format!("Include GWP: {}", settings.include_gwp) }</p>
            <p>{ format!("Include Opp MWP: {}", settings.include_opp_mwp) }</p>
            <p>{ format!("Include Opp GWP: {}", settings.include_opp_gwp) }</p>
        </div>
    }
}
