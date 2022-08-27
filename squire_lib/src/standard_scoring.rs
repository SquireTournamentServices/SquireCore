use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write as _},
};

use serde::{Deserialize, Serialize};

use crate::{
    identifiers::PlayerId,
    player_registry::PlayerRegistry,
    round::Round,
    round_registry::RoundRegistry,
    scoring::{Score, Standings},
    settings::StandardScoringSetting,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(C)]
/// The score type used by the standard scoring system
pub struct StandardScore {
    /// The number of match points a player has
    pub match_points: f64,
    /// The number of game points a player has
    pub game_points: f64,
    /// The match win percentage of a player
    pub mwp: f64,
    /// The game win percentage of a player
    pub gwp: f64,
    /// The average match win percentage of a player's opponents
    pub opp_mwp: f64,
    /// The average game win percentage of a player's opponents
    pub opp_gwp: f64,
    /// Whether or not match points should be considered
    pub include_match_points: bool,
    /// Whether or not game points should be considered
    pub include_game_points: bool,
    /// Whether or not match win percentage should be considered
    pub include_mwp: bool,
    /// Whether or not game win percentage should be considered
    pub include_gwp: bool,
    /// Whether or not opponent's match win percentage should be considered
    pub include_opp_mwp: bool,
    /// Whether or not opponent's game win percentage should be considered
    pub include_opp_gwp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A counter used to track player info while calculating scores
struct ScoreCounter {
    pub(crate) player: PlayerId,
    pub(crate) games: u64,
    pub(crate) game_wins: u64,
    pub(crate) game_losses: u64,
    pub(crate) game_draws: u64,
    pub(crate) rounds: u64,
    pub(crate) wins: u64,
    pub(crate) losses: u64,
    pub(crate) draws: u64,
    pub(crate) byes: u64,
    pub(crate) opponents: HashSet<PlayerId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[repr(C)]
/// The scoring stuct that uses the standard match point model
pub struct StandardScoring {
    match_win_points: f64,
    match_draw_points: f64,
    match_loss_points: f64,
    game_win_points: f64,
    game_draw_points: f64,
    game_loss_points: f64,
    bye_points: f64,
    include_byes: bool,
    include_match_points: bool,
    include_game_points: bool,
    include_mwp: bool,
    include_gwp: bool,
    include_opp_mwp: bool,
    include_opp_gwp: bool,
}

impl StandardScoring {
    /// Creates a new standard scorig system
    pub fn new() -> Self {
        StandardScoring {
            match_win_points: 3.0,
            match_draw_points: 1.0,
            match_loss_points: 0.0,
            game_win_points: 3.0,
            game_draw_points: 1.0,
            game_loss_points: 0.0,
            bye_points: 3.0,
            include_byes: true,
            include_match_points: true,
            include_game_points: true,
            include_mwp: true,
            include_gwp: true,
            include_opp_mwp: true,
            include_opp_gwp: true,
        }
    }

    fn new_score(&self) -> StandardScore {
        StandardScore::new(
            self.include_match_points,
            self.include_game_points,
            self.include_mwp,
            self.include_gwp,
            self.include_opp_mwp,
            self.include_opp_gwp,
        )
    }

    fn calculate_match_points_with_byes(&self, counter: &ScoreCounter) -> f64 {
        self.match_win_points * (counter.wins as f64)
            + self.match_draw_points * (counter.draws as f64)
            + self.match_loss_points * (counter.losses as f64)
            + self.bye_points * (counter.byes as f64)
    }

    fn calculate_match_points_without_byes(&self, counter: &ScoreCounter) -> f64 {
        self.match_win_points * (counter.wins as f64)
            + self.match_draw_points * (counter.draws as f64)
            + self.match_loss_points * (counter.losses as f64)
    }

    fn calculate_game_points(&self, counter: &ScoreCounter) -> f64 {
        self.game_win_points * (counter.game_wins as f64)
            + self.game_draw_points * (counter.game_draws as f64)
            + self.game_loss_points * (counter.game_losses as f64)
    }

    /// Updates a single scoring setting
    pub fn update_setting(&mut self, setting: StandardScoringSetting) {
        use StandardScoringSetting::*;
        match setting {
            MatchWinPoints(p) => {
                self.match_win_points = p;
            }
            MatchDrawPoints(p) => {
                self.match_draw_points = p;
            }
            MatchLossPoints(p) => {
                self.match_loss_points = p;
            }
            GameWinPoints(p) => {
                self.game_win_points = p;
            }
            GameDrawPoints(p) => {
                self.game_draw_points = p;
            }
            GameLossPoints(p) => {
                self.game_loss_points = p;
            }
            ByePoints(p) => {
                self.bye_points = p;
            }
            IncludeByes(b) => {
                self.include_byes = b;
            }
            IncludeMatchPoints(b) => {
                self.include_match_points = b;
            }
            IncludeGamePoints(b) => {
                self.include_game_points = b;
            }
            IncludeMwp(b) => {
                self.include_mwp = b;
            }
            IncludeGwp(b) => {
                self.include_gwp = b;
            }
            IncludeOppMwp(b) => {
                self.include_opp_mwp = b;
            }
            IncludeOppGwp(b) => {
                self.include_opp_gwp = b;
            }
        }
    }

    /// Calculates all the standing for the active players
    pub fn get_standings(
        &self,
        player_reg: &PlayerRegistry,
        round_reg: &RoundRegistry,
    ) -> Standings<StandardScore> {
        let mut counters: HashMap<PlayerId, ScoreCounter> = player_reg
            .players
            .iter()
            .filter(|(_, p)| p.can_play())
            .map(|(id, _)| (*id, ScoreCounter::new(*id)))
            .collect();
        for (_, round) in round_reg.rounds.iter() {
            if !round.is_certified() {
                continue;
            }
            if round.is_bye && !self.include_byes {
                continue;
            }
            for p in round.players.iter() {
                let counter = counters.get_mut(p).unwrap();
                counter.add_round(round)
            }
        }
        // We have tallied everyone's round results. Time to calculate everyone's scores
        let mut digest: HashMap<PlayerId, StandardScore> = HashMap::with_capacity(counters.len());
        for (id, counter) in &counters {
            let mut score = self.new_score();
            score.match_points = self.calculate_match_points_with_byes(counter);
            score.game_points = self.calculate_game_points(counter);
            // If your only round was a bye, your percentages stay at 0
            // This also filters out folks that haven't played a match yet
            if counter.rounds != counter.byes {
                score.mwp = score.match_points / (self.match_win_points * (counter.rounds as f64));
                score.gwp = score.game_points / (self.game_win_points * (counter.games as f64));
            }
            digest.insert(*id, score);
        }
        for (id, counter) in &counters {
            // If your only round was a bye, your percentages stay at 0
            // This also filters out folks that haven't played a match yet
            if counter.rounds == counter.byes {
                continue;
            }
            let mut opp_mp: f64 = 0.0;
            let mut opp_matches: u64 = 0;
            let mut opp_gp: f64 = 0.0;
            let mut opp_games: u64 = 0;
            for plyr in counter.opponents.iter().filter(|i| *i != id) {
                opp_mp += self.calculate_match_points_without_byes(&counters[plyr]);
                opp_matches += counters[plyr].rounds - counters[plyr].byes;
                opp_gp += self.calculate_game_points(&counters[plyr]);
                opp_games += counters[plyr].games;
            }
            digest.get_mut(id).unwrap().opp_mwp = opp_mp / (opp_matches as f64);
            digest.get_mut(id).unwrap().opp_gwp = opp_gp / (opp_games as f64);
        }
        let mut results: Vec<(PlayerId, StandardScore)> = digest.drain().collect();
        results.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        Standings::new(results)
    }
}

impl StandardScore {
    fn new(
        include_match_points: bool,
        include_game_points: bool,
        include_mwp: bool,
        include_gwp: bool,
        include_opp_mwp: bool,
        include_opp_gwp: bool,
    ) -> Self {
        StandardScore {
            match_points: 0.0,
            game_points: 0.0,
            mwp: 0.0,
            gwp: 0.0,
            opp_mwp: 0.0,
            opp_gwp: 0.0,
            include_match_points,
            include_game_points,
            include_mwp,
            include_gwp,
            include_opp_mwp,
            include_opp_gwp,
        }
    }
}

impl Score for StandardScore {}

impl Display for StandardScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.include_match_points
            | !self.include_game_points
            | !self.include_mwp
            | !self.include_gwp
            | !self.include_opp_mwp
            | !self.include_opp_gwp
        {
            return write!(f, "StandardScore {{ }}");
        }
        let mut digest = String::from("StandardScore {{ ");
        if self.include_match_points {
            write!(digest, " match points: {:2}, ", self.match_points)?;
        }
        if self.include_game_points {
            write!(digest, " game points: {:2}, ", self.game_points)?;
        }
        if self.include_mwp {
            write!(digest, " match win percent: {:3}, ", self.mwp)?;
        }
        if self.include_gwp {
            write!(digest, " game win percent: {:3}, ", self.gwp)?;
        }
        if self.include_opp_mwp {
            write!(digest, " opponent match win percent: {:3}, ", self.opp_mwp)?;
        }
        if self.include_opp_gwp {
            write!(digest, " opponent game win percent: {:3}, ", self.opp_gwp)?;
        }
        let l = digest.len();
        write!(f, "{} }}", &digest[..l - 2])
    }
}

impl ScoreCounter {
    fn new(player: PlayerId) -> Self {
        ScoreCounter {
            player,
            games: 0,
            game_wins: 0,
            game_losses: 0,
            game_draws: 0,
            rounds: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            byes: 0,
            opponents: HashSet::new(),
        }
    }

    fn add_round(&mut self, round: &Round) {
        match &round.winner {
            Some(winner) => {
                if winner == &self.player {
                    self.add_win(&round.players);
                } else {
                    self.add_loss(&round.players);
                }
            }
            None => {
                if round.is_bye {
                    self.add_bye();
                } else {
                    self.add_draw(&round.players);
                }
            }
        }
        for (p_id, count) in &round.results {
            if p_id == &self.player {
                self.game_wins += *count as u64;
            } else {
                self.game_losses += *count as u64;
            }
        }
    }

    fn add_win(&mut self, players: &HashSet<PlayerId>) {
        self.wins += 1;
        self.games += 1;
        self.opponents.extend(players.clone());
    }

    fn add_loss(&mut self, players: &HashSet<PlayerId>) {
        self.losses += 1;
        self.games += 1;
        self.opponents.extend(players.clone());
    }

    fn add_draw(&mut self, players: &HashSet<PlayerId>) {
        self.draws += 1;
        self.games += 1;
        self.opponents.extend(players.clone());
    }

    fn add_bye(&mut self) {
        self.byes += 1;
    }
}

impl Default for StandardScoring {
    fn default() -> Self {
        Self::new()
    }
}
