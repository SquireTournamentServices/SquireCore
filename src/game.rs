use crate::player::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameResult {
    Win,
    Draw,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub(crate) result: GameResult,
    pub(crate) winner: Option<PlayerId>,
}

impl Game {
    pub fn new_win(player: PlayerId) -> Self {
        Game {
            result: GameResult::Win,
            winner: Some(player),
        }
    }

    pub fn new_draw(player: PlayerId) -> Self {
        Game {
            result: GameResult::Draw,
            winner: None,
        }
    }

    pub fn get_winner(&self) -> Option<PlayerId> {
        self.winner
    }

    pub fn get_result(&self) -> GameResult {
        self.result.clone()
    }

    pub fn is_draw(&self) -> bool {
        self.result == GameResult::Draw
    }
}
