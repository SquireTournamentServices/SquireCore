use serde::{Deserialize, Serialize};

use crate::identifiers::PlayerId;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct for communicating new pairings information
pub struct Pairings {
    /// The players that are paired and their groupings
    pub paired: Vec<Vec<PlayerId>>,
    /// The players that aren't paired
    pub rejected: Vec<PlayerId>,
}
