use serde::{Deserialize, Serialize};

use crate::identifiers::PlayerId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pairings {
    pub paired: Vec<Vec<PlayerId>>,
    pub rejected: Vec<PlayerId>,
}
