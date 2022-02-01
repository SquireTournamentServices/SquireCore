pub use crate::scoring_system::{
    HashMap, PlayerRegistry, RoundRegistry, Score, ScoringSystem, Standings,
};

pub use crate::round::Round;

use uuid::Uuid;

use std::collections::HashSet;
use std::string::ToString;

#[derive(PartialEq, PartialOrd)]
pub struct StandardScore {
    pub(crate) match_points: f64,
    pub(crate) game_points: f64,
    pub(crate) mwp: f64,
    pub(crate) gwp: f64,
    pub(crate) opp_mwp: f64,
    pub(crate) opp_gwp: f64,
    pub(crate) include_match_points: bool,
    pub(crate) include_game_points: bool,
    pub(crate) include_mwp: bool,
    pub(crate) include_gwp: bool,
    pub(crate) include_opp_mwp: bool,
    pub(crate) include_opp_gwp: bool,
}

struct ScoreCounter {
    pub player: Uuid,
    pub games: u64,
    pub game_wins: u64,
    pub game_losses: u64,
    pub game_draws: u64,
    pub rounds: u64,
    pub wins: u64,
    pub losses: u64,
    pub draws: u64,
    pub byes: u64,
    pub opponents: HashSet<Uuid>,
}

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
}

impl ScoringSystem for StandardScoring {
    fn new() -> Self
    where
        Self: Sized,
    {
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

    fn update_settings(&mut self, settings: HashMap<String, String>) -> Result<(), ()> {
        todo!()
    }

    fn get_standings(&self, player_reg: &PlayerRegistry, round_reg: &RoundRegistry) -> Standings {
        let mut counters: HashMap<Uuid, ScoreCounter> = player_reg
            .iter()
            .filter(|(_, p)| p.can_play())
            .map(|(id, _)| (id.clone(), ScoreCounter::new(id.clone())))
            .collect();
        for (id, round) in round_reg.iter() {
            if !round.is_certified() {
                continue;
            }
            if round.is_bye && !self.include_byes {
                continue;
            }
            for p in round.players.iter().cloned() {
                let counter = counters.get_mut(&p).unwrap();
                counter.add_round(round)
            }
        }
        // We have tallied everyone's round results. Time to calculate everyone's scores
        let mut digest: Vec<(String, StandardScore)> = Vec::with_capacity(counters.len());
        /*
        mwp: f64,
        gwp: f64,
        opp_mwp: f64,
        opp_gwp: f64,
        */
        /*
        player: Uuid,
        games: u64,
        rounds: u64,
        opponents: HashSet<Uuid>,
        */
        for (_, counter) in counters {
            let mut score = self.new_score();
            score.match_points = self.match_win_points * (counter.wins as f64)
                + self.match_draw_points * (counter.draws as f64)
                + self.match_loss_points * (counter.losses as f64)
                + self.bye_points * (counter.byes as f64);
            score.game_points = self.game_win_points * (counter.game_wins as f64)
                + self.game_draw_points * (counter.game_draws as f64)
                + self.game_loss_points * (counter.game_losses as f64);
        }
        digest.sort_by(|(_, a), (_, b)| a.partial_cmp(&b).unwrap());
        Standings::new(
            digest
                .into_iter()
                .map(|(name, score)| (name, Box::new(score) as Box<dyn Score>))
                .collect::<Vec<(String, Box<dyn Score>)>>(),
        )
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

impl ToString for StandardScore {
    fn to_string(&self) -> String {
        if !self.include_match_points
            | !self.include_game_points
            | !self.include_mwp
            | !self.include_gwp
            | !self.include_opp_mwp
            | !self.include_opp_gwp
        {
            return String::new();
        }
        let mut digest = String::with_capacity(28);
        if self.include_match_points {
            digest += &format!("{:2}, ", self.match_points);
        }
        if self.include_game_points {
            digest += &format!("{:2}, ", self.game_points);
        }
        if self.include_mwp {
            digest += &format!("{:.3}, ", self.mwp);
        }
        if self.include_gwp {
            digest += &format!("{:.3}, ", self.gwp);
        }
        if self.include_opp_mwp {
            digest += &format!("{:.3}, ", self.opp_mwp);
        }
        if self.include_opp_gwp {
            digest += &format!("{:.3}, ", self.opp_gwp);
        }
        let l = digest.len();
        // Safety check: Since digest can only be empty when all condititions are false (a dumb
        // idea), check for this at the start. Otherwise, at least condition is true and contains
        // at least `, `.
        digest[..l - 2].to_string()
    }
}

impl ScoreCounter {
    fn new(player: Uuid) -> Self {
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
        match round.winner {
            Some(winner) => {
                if winner == self.player {
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
        for game in &round.games {
            if game.is_draw() {
                self.game_draws += 1;
            } else {
                if game.winner.unwrap() == self.player {
                    self.game_wins += 1;
                } else {
                    self.game_losses += 1;
                }
            }
        }
    }

    fn add_win(&mut self, players: &HashSet<Uuid>) {
        self.wins += 1;
        self.games += 1;
        self.opponents.extend(players);
    }

    fn add_loss(&mut self, players: &HashSet<Uuid>) {
        self.losses += 1;
        self.games += 1;
        self.opponents.extend(players);
    }

    fn add_draw(&mut self, players: &HashSet<Uuid>) {
        self.draws += 1;
        self.games += 1;
        self.opponents.extend(players);
    }

    fn add_bye(&mut self) {
        self.byes += 1;
        self.games += 1;
    }
}
