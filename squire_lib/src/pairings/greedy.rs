use std::collections::{HashMap, HashSet, VecDeque};

use crate::{identifiers::PlayerId, pairings::Pairings};

// TODO: PLEASE provide a better description
/// A simple, greedy algorithm that attempts to pair the first players first
pub fn greedy_pairings<Players>(
    plyrs: Players,
    opps: &HashMap<PlayerId, HashSet<PlayerId>>,
    match_size: usize,
    repair_tol: u64,
) -> Pairings
where
    Players: IntoIterator<Item = PlayerId>,
{
    let mut plyrs: VecDeque<_> = plyrs.into_iter().collect();
    let mut digest = Pairings {
        paired: Vec::with_capacity(plyrs.len() / match_size + 1),
        rejected: Vec::new(),
    };
    'outer: while plyrs.len() >= match_size {
        let Some(first) = plyrs.pop_front() else { break; };
        let mut id_buffer: Vec<PlayerId> = Vec::with_capacity(match_size);

        for plyr in &plyrs {
            let current_pairing = std::iter::once(&first).chain(id_buffer.iter());
            if valid_pairing(opps, current_pairing, plyr, repair_tol) {
                id_buffer.push(*plyr);
                if id_buffer.len() == match_size - 1 {
                    plyrs.retain(|p| !id_buffer.contains(p));
                    id_buffer.push(first);
                    digest.paired.push(id_buffer);
                    continue 'outer;
                }
            }
        }

        digest.rejected.push(first);
    }
    digest.rejected.extend(plyrs);
    digest
}

/// Checks to see if a player can be apart of a potential pairing
fn valid_pairing<'a>(
    past_opponents: &HashMap<PlayerId, HashSet<PlayerId>>,
    known: impl Iterator<Item = &'a PlayerId>,
    new: &PlayerId,
    repair_tol: u64,
) -> bool {
    past_opponents.get(new).map_or(true, |opps| {
        known.filter(|p| opps.contains(p)).count() as u64 <= repair_tol
    })
}
