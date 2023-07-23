use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use squire_sdk::{
    model::{
        players::{Player, PlayerId},
        rounds::{Round, RoundId, RoundStatus},
        tournament::Tournament,
    },
    sync::TournamentManager,
};
use yew::{html, Callback, Html};

use super::{players::SelectedPlayerMessage, rounds::RoundSummary};
use crate::tournament::players::SubviewInfo;

/// The set of data needed by the UI to display a player. Should be capable of rendering itself in
/// HTML.
///
/// NOTE: Under construction
#[derive(Debug, PartialEq, Clone)]
pub struct PlayerProfile {
    pub id: PlayerId,
    pub name: String,
    pub gamer_tag: Option<String>,
    pub can_play: bool,
    pub rounds: Vec<RoundSummary>,
}
impl PlayerProfile {
    pub fn new(plyr: &Player, t: &TournamentManager) -> Self {
        let mut to_return = Self {
            id: plyr.id,
            name: plyr.name.clone(),
            gamer_tag: plyr.game_name.clone(),
            can_play: plyr.can_play(),
            rounds: t
                .get_player_rounds(&plyr.id.into())
                .unwrap()
                .iter()
                .map(|r| RoundSummary::new(*r))
                .collect(),
        };
        to_return.rounds.sort_by_cached_key(|r| r.match_number);
        to_return.rounds.sort_by_cached_key(|r| r.status);
        to_return
    }

    pub fn view(&self, process: Callback<SelectedPlayerMessage>) -> Html {
        let id = self.id.clone();
        let cb = process.clone();
        let dropplayer = move |_| {
            cb.emit(SelectedPlayerMessage::DropPlayer(id));
        };
        let list = self
            .rounds
            .iter()
            .cloned()
            .map(|r| {
                let cb = process.clone();
                html! {
                    <tr onclick = {move |_| cb.emit(SelectedPlayerMessage::SubviewSelected(SubviewInfo::Round(r.id)))}>
                        <td>{ r.match_number }</td>
                        <td>{ r.table_number }</td>
                        <td>{ r.status }</td>
                    </tr>
                }
            })
            .collect::<Html>();
        html! {
            <>
                <>
                    <>
                        <h4>{ self.name.as_str() }</h4>
                        <p>{ format!("Gamertag : {}", self.gamer_tag.clone().unwrap_or_default() ) }</p>
                        <p>{ format!("Can play : {}", self.can_play) }</p>
                        <p>{ format!("Rounds : {}", self.rounds.len()) }</p>
                    </>
                </>
                <table class="table">
                    <thead>
                        <tr>
                            <th>{ "Round" }</th>
                            <th>{ "Table" }</th>
                            <th>{ "Status" }</th>
                        </tr>
                    </thead>
                    <tbody> { list } </tbody>
                </table>
                <button type="button" class="btn btn-danger" data-bs-toggle="modal" data-bs-target="#killModal">
                {"Drop player"}
                </button>
                <div class="modal fade" id="killModal" tabindex="-1" aria-labelledby="killModalLabel" aria-hidden="true">
                  <div class="modal-dialog">
                    <div class="modal-content">
                      <div class="modal-header">
                        <h1 class="modal-title fs-5" id="exampleModalLabel">{"Kill round confirmation"}</h1>
                        <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                      </div>
                      <div class="modal-body">
                        {"Do you REALLY want to drop this player?"}
                      </div>
                      <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Go back"}</button>
                        <button type="button" onclick={ dropplayer } class="btn btn-primary" data-bs-dismiss="modal">{"Drop Player"}</button>
                      </div>
                    </div>
                  </div>
                </div>
            </>
        }
    }
}

/// The set of data needed by the UI to display a round. Should be capable of rendering itself in
/// HTML.
///
/// NOTE: Under construction
#[derive(Debug, PartialEq, Clone)]
pub struct RoundProfile {
    pub id: RoundId,
    pub order: Vec<PlayerId>,
    pub player_names: HashMap<PlayerId, String>,
    pub timer: DateTime<Utc>,
    pub status: RoundStatus,
    pub results: HashMap<PlayerId, u32>,
    pub draws: u32,
    pub confirmations: HashSet<PlayerId>,
    pub length: std::time::Duration,
    pub extensions: std::time::Duration,
}
impl RoundProfile {
    pub fn new(tourn: &Tournament, rnd: &Round) -> Self {
        Self {
            id: rnd.id,
            status: rnd.status,
            order: rnd.players.clone(),
            player_names: rnd
                .players
                .iter()
                .filter_map(|p| {
                    tourn
                        .player_reg
                        .players
                        .get(p)
                        .map(|plyr| (*p, plyr.name.clone()))
                })
                .collect(), // This is not a Vec<(PlayerId, String)>. This is a HashMap
            length: rnd.length,
            extensions: rnd.extension,
            timer: rnd.timer,
            results: rnd.results.clone(),
            draws: rnd.draws,
            confirmations: rnd.confirmations.clone(),
        }
    }

    pub fn view(&self) -> Html {
        // TODO: Remove unwrap here
        let dur_left = ChronoDuration::from_std(self.length + self.extensions).unwrap()
            - (Utc::now() - self.timer);
        let list = self
            .order
            .iter()
            .map(|pid| {
                let player_name = self.player_names.get(pid).cloned().unwrap_or_default();
                let player_wins = self.results.get(pid).cloned().unwrap_or_default();
                let player_confirm = self.confirmations.get(pid).is_some();
                html! {
                    <tr>
                        <td>{ player_name }</td>
                        <td>{ player_wins }</td>
                        <td>{ player_confirm }</td>
                    </tr>
                }
            })
            .collect::<Html>();
        html! {
            <>
            <p>
            { pretty_print_duration(dur_left) }
            </p>
            <table class="table">
            <thead>
                <tr>
                    <th>{ "Name" }</th>
                    <th>{ "Wins" }</th>
                    <th>{ "Status" }</th>
                </tr>
            </thead>
            <tbody> { list } </tbody>
            </table>
            </>
        }
    }
}

/// Shows a duration as a formatted string.
///
/// NOTE: Under construction
fn pretty_print_duration(dur: ChronoDuration) -> String {
    let hours = dur.num_hours();
    let mins = dur.num_minutes().abs();
    let secs = dur.num_seconds().abs();
    if hours >= 0 {
        format!("Time left: {}:{}:{}", hours.abs(), mins % 60, secs % 60)
    } else {
        format!("Over time: {}:{}:{}", hours.abs(), mins % 60, secs % 60)
    }
}
