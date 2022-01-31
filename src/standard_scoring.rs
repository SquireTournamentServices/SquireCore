pub use crate::scoring_system::{
    HashMap, PlayerRegistry, RoundRegistry, Score, ScoringSystem, Standings,
};

use uuid::Uuid;

use std::string::ToString;

#[derive(PartialEq, PartialOrd)]
pub struct StandardScore {
    pub(crate) wins: u64,
    pub(crate) losses: u64,
    pub(crate) draws: u64,
    pub(crate) byes: u64,
    pub(crate) games: u64,
    pub(crate) include_match_points: bool,
    pub(crate) include_game_points: bool,
    pub(crate) include_mwp: bool,
    pub(crate) include_gwp: bool,
    pub(crate) include_opp_mwp: bool,
    pub(crate) include_opp_gwp: bool,
}

pub struct StandardScoring {
    match_win_points: i64,
    match_draw_points: i64,
    match_loss_points: i64,
    game_win_points: i64,
    game_draw_points: i64,
    game_loss_points: i64,
    bye_points: i64,
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
            match_win_points: 3,
            match_draw_points: 1,
            match_loss_points: 0,
            game_win_points: 3,
            game_draw_points: 1,
            game_loss_points: 0,
            bye_points: 3,
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
        let mut id_and_scores: HashMap<Uuid, StandardScore> = player_reg
            .iter()
            .filter(|(_, p)| p.can_play())
            .map(|(id, _)| (id.clone(), self.new_score()))
            .collect();
        for (id,round) in round_reg.iter() {
            if !round.is_certified() {
                continue;
            }
            if round.is_bye && !self.include_byes {
                continue;
            }
            if round.is_bye {
                let s = id_and_scores.get_mut(&round.winner.unwrap());
            } else {
            }
        }
        todo!()
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
            wins: 0,
            losses: 0,
            draws: 0,
            byes: 0,
            games: 0,
            include_match_points,
            include_game_points,
            include_mwp,
            include_gwp,
            include_opp_mwp,
            include_opp_gwp,
        }
    }
    
    fn add_win(&mut self) {
        self.wins += 1;
        self.games += 1;
    }
    
    fn add_loss(&mut self) {
        self.losses += 1;
        self.games += 1;
    }
    
    fn add_draw(&mut self) {
        self.draws += 1;
        self.games += 1;
    }
    
    fn add_bye(&mut self) {
        self.byes += 1;
        self.games += 1;
    }
    
    fn get_match
}

impl Score for StandardScore {}

impl ToString for StandardScore { fn to_string(&self) -> String { if !self.include_match_points
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
