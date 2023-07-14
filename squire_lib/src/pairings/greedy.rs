use std::collections::{HashMap, HashSet, VecDeque};

use crate::{identifiers::PlayerId, pairings::Pairings};

// TODO: PLEASE provide a better description
/// A pairing algorithm that attempts to pair players greedily, consuming players as soon as
/// possible to build matches.
///
/// Match building begins from the front of the list: Players are picked from `plyrs` based on
/// whether or not they are compatible with the other players that have already been picked (so the
/// first time, the player at the start of the list will always be picked). If a full pairing has
/// been built, those players in that match are removed from the list and placed into a pairing.
/// Otherwise, the first player is removed and rejected, and the process begins again with the
/// modified `plyrs`. This process continues until `plyrs` has been depleted.
///
/// # Panics
///
/// Will panics when `match_size` is zero.
pub fn greedy_pairings<Players>(
    plyrs: Players,
    opps: &HashMap<PlayerId, HashSet<PlayerId>>,
    match_size: usize,
    repair_tol: u64,
) -> Pairings
    where
        Players: IntoIterator<Item=PlayerId>,
{
    let mut plyrs: VecDeque<_> = plyrs.into_iter().collect();
    let mut digest = Pairings {
        paired: Vec::with_capacity(plyrs.len() / match_size + 1),
        rejected: Vec::new(),
    };
    'outer: while plyrs.len() >= match_size {
        let Some(first) = plyrs.pop_front() else {
            break;
        };
        let mut id_buffer: Vec<PlayerId> = Vec::with_capacity(match_size);

        for plyr in &plyrs {
            let current_pairing = std::iter::once(&first).chain(id_buffer.iter());
            if valid_pairing(opps, current_pairing, plyr, repair_tol) {
                id_buffer.push(*plyr);
                if id_buffer.len() == match_size - 1 {
                    plyrs.retain(|p| !id_buffer.contains(p));
                    id_buffer.insert(0, first);
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
    known: impl Iterator<Item=&'a PlayerId>,
    new: &PlayerId,
    repair_tol: u64,
) -> bool {
    past_opponents.get(new).map_or(true, |opps| {
        known.filter(|p| opps.contains(p)).count() as u64 <= repair_tol
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::{identifiers::PlayerId, pairings::Pairings};

    #[test]
    fn trivial() {
        let players: Vec<_> = std::iter::repeat_with(|| PlayerId::new(Uuid::new_v4()))
            .take(4)
            .collect();
        let opponents = HashMap::new();

        let pairings = super::greedy_pairings(players.iter().cloned(), &opponents, 4, 0);
        let Pairings {
            mut paired,
            rejected,
        } = pairings;
        assert_eq!(
            paired.len(),
            1,
            "Exactly one pairing should have been produced"
        );
        assert!(
            rejected.is_empty(),
            "No player should have been rejected from a pairing"
        );

        let pair = paired.remove(0);
        drop(paired);
        assert_eq!(pair.len(), 4, "Pairing should contain all four players");
        for p in players {
            assert!(
                pair.contains(&p),
                "All players in the registry should be included in the pairing"
            );
        }
    }

    #[test]
    fn exclude_one() {
        let players: Vec<_> = std::iter::repeat_with(|| PlayerId::new(Uuid::new_v4()))
            .take(5)
            .collect();
        let opponents = HashMap::new();

        let pairings = super::greedy_pairings(players.iter().cloned(), &opponents, 4, 0);
        let Pairings {
            mut paired,
            rejected,
        } = pairings;
        assert_eq!(
            paired.len(),
            1,
            "Exactly one pairing should have been produced"
        );
        assert_eq!(
            rejected.len(),
            1,
            "Exactly one player should have been rejected from pairing"
        );

        assert_eq!(
            &rejected[0], &players[4],
            "The last player should have been rejected"
        );

        let pair = paired.remove(0);
        drop(paired);
        assert_eq!(pair.len(), 4, "Pairing should contain four players");
        for player in pair {
            assert!(
                players[..4].contains(&player),
                "Pairing should only contain the first four players"
            )
        }
    }

    #[test]
    fn exclude_second() {
        let players: Vec<_> = std::iter::repeat_with(|| PlayerId::new(Uuid::new_v4()))
            .take(5)
            .collect();
        let opponents = [
            (players[0], [players[1]].into_iter().collect()),
            (players[1], [players[0]].into_iter().collect()),
        ]
            .into_iter()
            .collect();

        let pairings = super::greedy_pairings(players.iter().cloned(), &opponents, 4, 0);
        let Pairings {
            mut paired,
            rejected,
        } = pairings;
        assert_eq!(
            paired.len(),
            1,
            "Exactly one pairing should have been produced"
        );
        assert_eq!(
            rejected.len(),
            1,
            "Exactly one player should have been rejected from pairing"
        );

        assert_eq!(
            &rejected[0], &players[1],
            "The second player should have been rejected"
        );

        let pair = paired.remove(0);
        assert_eq!(pair.len(), 4, "Pairing should contain four players");
        for player in pair {
            assert!(
                [players[0], players[2], players[3], players[4]].contains(&player),
                "Pairing should only contain all except the second player"
            )
        }
    }

    #[test]
    fn pair_pairs() {
        let players: Vec<_> = std::iter::repeat_with(|| PlayerId::new(Uuid::new_v4()))
            .take(8)
            .collect();

        let pairings = super::greedy_pairings(players.iter().cloned(), &HashMap::new(), 2, 0);
        let Pairings { paired, rejected } = pairings;
        assert!(
            rejected.is_empty(),
            "No player should have been rejected from a pairing (first pairing)"
        );

        let [pair1, pair2, pair3, pair4] = &paired[..] else {
            panic!("Exactly four pairings should have been produced (first pairing)")
        };

        assert_eq!(pair1, &vec![players[0], players[1]]);
        assert_eq!(pair2, &vec![players[2], players[3]]);
        assert_eq!(pair3, &vec![players[4], players[5]]);
        assert_eq!(pair4, &vec![players[6], players[7]]);

        let opponents = paired
            .into_iter()
            .flat_map(|pair| [(pair[0], pair[1]), (pair[1], pair[0])])
            .map(|(a, b)| (a, [b].into_iter().collect()))
            .collect();
        let pairings = super::greedy_pairings(players.iter().cloned(), &opponents, 2, 0);
        let Pairings { paired, rejected } = pairings;
        assert!(
            rejected.is_empty(),
            "No player should have been rejected from a pairing (second pairing)"
        );

        let [pair1, pair2, pair3, pair4] = &paired[..] else {
            panic!("Exactly four pairings should have been produced (second pairing)")
        };

        assert_eq!(pair1, &vec![players[0], players[2]]);
        assert_eq!(pair2, &vec![players[1], players[3]]);
        assert_eq!(pair3, &vec![players[4], players[6]]);
        assert_eq!(pair4, &vec![players[5], players[7]]);
    }

    #[test]
    fn pair_quads() {
        let players: Vec<_> = std::iter::repeat_with(|| PlayerId::new(Uuid::new_v4()))
            .take(16)
            .collect();

        let pairings = super::greedy_pairings(players.iter().cloned(), &HashMap::new(), 4, 0);
        let Pairings { paired, rejected } = pairings;
        assert!(
            rejected.is_empty(),
            "No player should have been rejected from a pairing (first pairing)"
        );

        let [pair1, pair2, pair3, pair4] = &paired[..] else {
            panic!("Exactly four pairings should have been produced (first pairing)")
        };

        assert_eq!(pair1, &players[0..4].to_vec());
        assert_eq!(pair2, &players[4..8].to_vec());
        assert_eq!(pair3, &players[8..12].to_vec());
        assert_eq!(pair4, &players[12..16].to_vec());

        let opponents = paired
            .into_iter()
            .flat_map(|pair| {
                [
                    (pair[0], [pair[1], pair[2], pair[3]]),
                    (pair[1], [pair[0], pair[2], pair[3]]),
                    (pair[2], [pair[0], pair[1], pair[3]]),
                    (pair[3], [pair[0], pair[1], pair[2]]),
                ]
            })
            .map(|(a, b)| (a, b.into_iter().collect()))
            .collect();

        let pairings = super::greedy_pairings(players.iter().cloned(), &opponents, 4, 0);
        let Pairings { paired, rejected } = pairings;
        assert!(
            rejected.is_empty(),
            "No player should have been rejected from a pairing (second pairing)"
        );

        let [pair1, pair2, pair3, pair4] = &paired[..] else {
            panic!("Exactly four pairings should have been produced (second pairing)")
        };

        assert_eq!(
            pair1,
            &vec![players[0], players[4], players[8], players[12]]
        );
        assert_eq!(
            pair2,
            &vec![players[1], players[5], players[9], players[13]]
        );
        assert_eq!(
            pair3,
            &vec![players[2], players[6], players[10], players[14]]
        );
        assert_eq!(
            pair4,
            &vec![players[3], players[7], players[11], players[15]]
        );
    }
}
