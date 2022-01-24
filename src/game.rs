use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameResult {
    Win,
    Draw,
}

#[derive(Debug)]
pub struct Game {
    pub(crate) result: GameResult,
    pub(crate) winner: Option<Uuid>,
}

impl Game {
    pub fn new_win(player: Uuid) -> Self {
        Game {
            result: GameResult::Win,
            winner: Some(player),
        }
    }
    pub fn new_draw(player: Uuid) -> Self {
        Game {
            result: GameResult::Draw,
            winner: None,
        }
    }
    pub fn get_winner(&self) -> Option<Uuid> {
        self.winner.clone()
    }
    pub fn get_result(&self) -> GameResult {
        self.result.clone()
    }
    pub fn is_draw(&self) -> bool {
        self.result == GameResult::Draw
    }
}
