
use crate::game::Game;

use uuid::Uuid;

use std::{collections::HashSet, io::BufRead, ops::RangeBounds};

pub enum RoundStatus {
    Open,
    Uncertified,
    Certified,
    Dead,
}

// This struct should be able to handle N many games, unlike the Python equiv.
pub struct Round {
    uuid: Uuid,
    match_number: u64,
    players: HashSet<Uuid>,
    confirmations: HashSet<Uuid>,
    games: Vec<Game>,
    status: RoundStatus,
    winner: Option<Uuid>,
    length: u64,
    extension: u64,
    is_bye: bool,
}

impl Round {
    pub fn new(match_num: u64, len: u64) -> Self {
        Round {
            uuid: Uuid::new_v4(),
            match_number: match_num,
            players: HashSet::with_capacity(4),
            confirmations: HashSet::with_capacity(4),
            games: Vec::with_capacity(3),
            status: RoundStatus::Open,
            winner: None,
            length: len,
            extension: 0,
            is_bye: false,
        }
    }
    pub fn add_player(&mut self, player: Uuid) -> () {
        self.players.insert(player);
    }
    pub fn record_game(&mut self, game: Game) -> Result<(), ()> {
        if !game.is_draw() {
            // Safty check: A game that isn't a draw (i.e. someone won) can not be without a winner
            if self.players.contains(&game.get_winner().unwrap()) {
                Err(())
            } else {
                self.games.push(game);
                self.confirmations.clear();
                Ok(())
            }
        } else {
            self.confirmations.clear();
            Ok(())
        }
    }
    pub fn confirm_round(&mut self, player: Uuid) -> Result<RoundStatus, ()> {
        if !self.players.contains(&player) {
            Err(())
        } else {
            self.confirmations.insert(player);
            if self.confirmations.len() == self.players.len() {
                Ok(RoundStatus::Certified)
            } else {
                Ok(RoundStatus::Uncertified)
            }
        }
    }
    pub fn kill_round(&mut self) -> () {
        self.status = RoundStatus::Dead;
    }
    pub fn record_bye(&mut self) -> Result<(),()> {
        if self.players.len() != 1 {
            Err(())
        } else {
            self.is_bye = true;
            Ok(())
        }
    }
    pub fn clear_game_record(&mut self) -> () {
        self.games.clear();
    }
}
