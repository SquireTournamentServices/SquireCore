use serde::{Deserialize, Serialize};

use crate::swiss_pairings::PlayerId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pairings {
    pub paired: Vec<Vec<PlayerId>>,
    pub rejected: Vec<PlayerId>,
}
