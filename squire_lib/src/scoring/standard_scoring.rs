use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write as _},
};

use serde::{Deserialize, Serialize};

use crate::{
    identifiers::PlayerId,
    players::PlayerRegistry,
    r64,
    rounds::{Round, RoundRegistry},
    scoring::{Score, Standings},
    settings::StandardScoringSetting,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
#[repr(C)]
/// The score type used by the standard scoring system
pub struct StandardScore {
    /// The number of match points a player has
    pub match_points: r64,
    /// The number of game points a player has
    pub game_points: r64,
    /// The match win percentage of a player
    pub mwp: r64,
    /// The game win percentage of a player
    pub gwp: r64,
    /// The average match win percentage of a player's opponents
    pub opp_mwp: r64,
    /// The average game win percentage of a player's opponents
    pub opp_gwp: r64,
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
    pub(crate) games: i32,
    pub(crate) game_wins: i32,
    pub(crate) game_losses: i32,
    pub(crate) game_draws: i32,
    pub(crate) rounds: i32,
    pub(crate) wins: i32,
    pub(crate) losses: i32,
    pub(crate) draws: i32,
    pub(crate) byes: i32,
    pub(crate) opponents: HashSet<PlayerId>,
}

#[allow(missing_docs)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
/// The scoring stuct that uses the standard match point model
pub struct StandardScoring {
    pub match_win_points: r64,
    pub match_draw_points: r64,
    pub match_loss_points: r64,
    pub game_win_points: r64,
    pub game_draw_points: r64,
    pub game_loss_points: r64,
    pub bye_points: r64,
    pub include_byes: bool,
    pub include_match_points: bool,
    pub include_game_points: bool,
    pub include_mwp: bool,
    pub include_gwp: bool,
    pub include_opp_mwp: bool,
    pub include_opp_gwp: bool,
}

impl StandardScoring {
    /// Creates a new standard scorig system
    pub fn new() -> Self {
        StandardScoring {
            match_win_points: r64::from_integer(3),
            match_draw_points: r64::from_integer(1),
            match_loss_points: r64::from_integer(0),
            game_win_points: r64::from_integer(3),
            game_draw_points: r64::from_integer(1),
            game_loss_points: r64::from_integer(0),
            bye_points: r64::from_integer(3),
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

    fn calculate_match_points_with_byes(&self, counter: &ScoreCounter) -> r64 {
        self.match_win_points * counter.wins
            + self.match_draw_points * counter.draws
            + self.match_loss_points * counter.losses
            + self.bye_points * counter.byes
    }

    fn calculate_match_points_without_byes(&self, counter: &ScoreCounter) -> r64 {
        self.match_win_points * counter.wins
            + self.match_draw_points * counter.draws
            + self.match_loss_points * counter.losses
    }

    fn calculate_game_points(&self, counter: &ScoreCounter) -> r64 {
        self.game_win_points * counter.game_wins
            + self.game_draw_points * counter.game_draws
            + self.game_loss_points * counter.game_losses
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
            .keys()
            .map(|id| (*id, ScoreCounter::new(*id)))
            .collect();
        round_reg
            .rounds
            .values()
            .filter(|r| r.is_certified())
            .filter(|r| !r.is_bye() || self.include_byes)
            .flat_map(|r| r.players.iter().map(move |p| (p, r)))
            .for_each(|(p, r)| {
                counters.entry(*p).and_modify(|c| c.add_round(r));
            });
        // We have tallied everyone's round results. Time to calculate everyone's scores
        let mut digest: HashMap<PlayerId, StandardScore> = HashMap::with_capacity(counters.len());
        for (id, counter) in &counters {
            let mut score = self.new_score();
            score.match_points = self.calculate_match_points_with_byes(counter);
            score.game_points = self.calculate_game_points(counter);
            // If your only round was a bye, your percentages stay at 0
            // This also filters out folks that haven't played a match yet
            if counter.rounds != counter.byes {
                score.mwp = score.match_points / (self.match_win_points * counter.rounds);
                score.gwp = score.game_points / (self.game_win_points * counter.games);
            }
            digest.insert(*id, score);
        }
        for (id, counter) in &counters {
            // If your only round was a bye, your percentages stay at 0
            // This also filters out folks that haven't played a match yet
            if counter.rounds == counter.byes {
                continue;
            }
            let mut opp_mp: r64 = Default::default();
            let mut opp_matches: i32 = 0;
            let mut opp_gp: r64 = Default::default();
            let mut opp_games: i32 = 0;
            for plyr in counter.opponents.iter().filter(|i| *i != id) {
                opp_mp += self.calculate_match_points_without_byes(&counters[plyr]);
                opp_matches += counters[plyr].rounds - counters[plyr].byes;
                opp_gp += self.calculate_game_points(&counters[plyr]);
                opp_games += counters[plyr].games;
            }
            digest.get_mut(id).unwrap().opp_mwp = if opp_matches == 0 {
                Default::default()
            } else {
                opp_mp / (self.match_win_points * opp_matches)
            };
            digest.get_mut(id).unwrap().opp_gwp = if opp_games == 0 {
                Default::default()
            } else {
                opp_gp / (self.game_win_points * opp_games)
            };
        }
        let mut results: Vec<(PlayerId, StandardScore)> = digest
            .drain()
            .filter(|(p, _)| player_reg.get_player(p).unwrap().can_play())
            .collect();
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
            match_points: Default::default(),
            game_points: Default::default(),
            mwp: Default::default(),
            gwp: Default::default(),
            opp_mwp: Default::default(),
            opp_gwp: Default::default(),
            include_match_points,
            include_game_points,
            include_mwp,
            include_gwp,
            include_opp_mwp,
            include_opp_gwp,
        }
    }
}

impl Score for StandardScore {
    fn primary_score(&self) -> r64 {
        self.match_points
    }
}

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
        self.rounds += 1;
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
                self.game_wins += *count as i32;
            } else {
                self.game_losses += *count as i32;
            }
        }
    }

    fn add_win(&mut self, players: &[PlayerId]) {
        self.wins += 1;
        self.games += 1;
        self.opponents.extend(players);
    }

    fn add_loss(&mut self, players: &[PlayerId]) {
        self.losses += 1;
        self.games += 1;
        self.opponents.extend(players);
    }

    fn add_draw(&mut self, players: &[PlayerId]) {
        self.draws += 1;
        self.games += 1;
        self.opponents.extend(players);
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
