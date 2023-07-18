use squire_sdk::model::settings::{
    ScoringSettingsTree, ScoringStyleSettingsTree, SettingsTree, StandardScoringSetting,
    TournamentSetting,
};
use yew::prelude::*;

use super::panel::{make_panel, SettingPanel};

pub struct ScoringSettings {
    match_win_points: SettingPanel,
    match_draw_points: SettingPanel,
    match_loss_points: SettingPanel,
    game_win_points: SettingPanel,
    game_draw_points: SettingPanel,
    game_loss_points: SettingPanel,
    bye_points: SettingPanel,
    include_byes: SettingPanel,
    include_match_points: SettingPanel,
    include_game_points: SettingPanel,
    include_mwp: SettingPanel,
    include_gwp: SettingPanel,
    include_opp_mwp: SettingPanel,
    include_opp_gwp: SettingPanel,
    current: ScoringSettingsTree,
    to_change: ScoringSettingsTree,
}

impl ScoringSettings {
    pub(crate) fn get_changes(&self) -> impl Iterator<Item = TournamentSetting> {
        self.to_change.diff(&self.current).map(Into::into)
    }
}

impl ScoringSettings {
    pub fn new(emitter: Callback<TournamentSetting>, tree: ScoringSettingsTree) -> Self {
        Self {
            match_win_points: make_panel(
                &emitter,
                "Match Win Points",
                StandardScoringSetting::MatchWinPoints,
            ),
            match_draw_points: make_panel(
                &emitter,
                "Match Draw Points:",
                StandardScoringSetting::MatchDrawPoints,
            ),
            match_loss_points: make_panel(
                &emitter,
                "Match Loss Points:",
                StandardScoringSetting::MatchLossPoints,
            ),
            game_win_points: make_panel(
                &emitter,
                "Game Win Points",
                StandardScoringSetting::GameWinPoints,
            ),
            game_draw_points: make_panel(
                &emitter,
                "Game Draw Points",
                StandardScoringSetting::GameDrawPoints,
            ),
            game_loss_points: make_panel(
                &emitter,
                "Game Loss Points",
                StandardScoringSetting::GameLossPoints,
            ),
            bye_points: make_panel(&emitter, "Bye Points", StandardScoringSetting::ByePoints),
            include_byes: make_panel(
                &emitter,
                "Include Byes",
                StandardScoringSetting::IncludeByes,
            ),
            include_match_points: make_panel(
                &emitter,
                "Include Match Points",
                StandardScoringSetting::IncludeMatchPoints,
            ),
            include_game_points: make_panel(
                &emitter,
                "Include Game Points",
                StandardScoringSetting::IncludeGamePoints,
            ),
            include_mwp: make_panel(
                &emitter,
                "Include Match Win Percent",
                StandardScoringSetting::IncludeMwp,
            ),
            include_gwp: make_panel(
                &emitter,
                "Include Game Win Percent",
                StandardScoringSetting::IncludeGwp,
            ),
            include_opp_mwp: make_panel(
                &emitter,
                "Include Opponent MWP",
                StandardScoringSetting::IncludeOppMwp,
            ),
            include_opp_gwp: make_panel(
                &emitter,
                "Include Opponent GWP",
                StandardScoringSetting::IncludeOppGwp,
            ),
            current: tree.clone(),
            to_change: tree,
        }
    }

    /*
    pub fn update(&mut self, setting: ScoringSetting) -> bool {
        let _ = self.to_change.update(setting);
        false
    }
    */

    pub fn view(&self) -> Html {
        #[allow(irrefutable_let_patterns)]
        let ScoringStyleSettingsTree::Standard(style) = &self.current.style else { panic!() };
        html! {
            <div>
                <h2>{ "Scoring Settings:" }</h2>
                <p> { self.match_win_points.view(style.match_win_points) }</p>
                <p> { self.match_draw_points.view(style.match_draw_points) }</p>
                <p> { self.match_loss_points.view(style.match_loss_points) }</p>
                <p> { self.game_win_points.view(style.game_win_points) }</p>
                <p> { self.game_draw_points.view(style.game_draw_points) }</p>
                <p> { self.game_loss_points.view(style.game_loss_points) }</p>
                <p> { self.bye_points.view(style.bye_points) }</p>
                <p> { self.include_byes.view(style.include_byes) }</p>
                <p> { self.include_match_points.view(style.include_match_points) }</p>
                <p> { self.include_game_points.view(style.include_game_points) }</p>
                <p> { self.include_mwp.view(style.include_mwp) }</p>
                <p> { self.include_gwp.view(style.include_gwp) }</p>
                <p> { self.include_opp_mwp.view(style.include_opp_mwp) }</p>
                <p> { self.include_opp_gwp.view(style.include_opp_gwp) }</p>
            </div>
        }
    }
}
