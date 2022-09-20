use std::collections::{HashMap, HashSet};

use crate::{identifiers::PlayerId, pairings::Pairings};

// TODO: PLEASE provide a better description
/// A simple, greedy algorithm that attempts to pair the first players first
pub fn greedy_pairings(
    mut plyrs: Vec<PlayerId>,
    opps: &HashMap<PlayerId, HashSet<PlayerId>>,
    match_size: usize,
    repair_tol: u64,
) -> Pairings {
    let mut digest = Pairings {
        paired: Vec::with_capacity(plyrs.len() / match_size + 1),
        rejected: Vec::new(),
    };
    while plyrs.len() >= match_size {
        let mut index_buffer: Vec<usize> = Vec::with_capacity(match_size);
        let mut id_buffer: Vec<PlayerId> = Vec::with_capacity(match_size);
        index_buffer.push(0);
        id_buffer.push(plyrs[0]);
        for (i, _) in plyrs.iter().enumerate().skip(1) {
            if valid_pairing(opps, &id_buffer, &plyrs[i], repair_tol) {
                index_buffer.push(i);
                id_buffer.push(plyrs[i]);
                if index_buffer.len() == match_size {
                    break;
                }
            }
        }
        if index_buffer.len() == match_size {
            let mut pairing: Vec<PlayerId> = Vec::with_capacity(match_size);
            for (count, i) in index_buffer.iter().enumerate() {
                let id = plyrs.remove(i - count);
                pairing.push(id);
            }
            digest.paired.push(pairing);
        } else {
            digest.rejected.push(plyrs.pop().unwrap());
        }
    }
    digest.rejected.extend_from_slice(&plyrs);
    digest
}

/// Checks to see if a player can be apart of a potential pairing
fn valid_pairing(
    past_opponents: &HashMap<PlayerId, HashSet<PlayerId>>,
    known: &[PlayerId],
    new: &PlayerId,
    repair_tol: u64,
) -> bool {
    if let Some(opps) = past_opponents.get(new) {
        known.iter().filter(|p| opps.contains(p)).count() as u64 <= repair_tol
    } else {
        true
    }
}
