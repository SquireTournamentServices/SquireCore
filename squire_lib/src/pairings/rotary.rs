use std::{
    collections::{HashMap, HashSet},
    vec::IntoIter,
};

use itertools::{Combinations, Itertools};

use crate::{
    identifiers::PlayerId,
    pairings::{count_opps, Pairings},
};

#[allow(unused)]
/// The branching pairings impl
pub fn rotary_pairings(
    plyrs: Vec<PlayerId>,
    opps: &HashMap<PlayerId, HashSet<PlayerId>>,
    match_size: usize,
    repair_tol: u64,
) -> Pairings {
    let mut digest = process(plyrs.into_iter(), match_size, opps);
    let mut count = 0;
    while !digest.is_valid(opps, repair_tol) && count < 25 {
        count += 1;
        let plyrs = digest.paired.into_iter().flat_map(|p| p.into_iter());
        let temp = match count % 2 == 0 {
            true => process(plyrs, match_size, opps),
            false => process(plyrs.rev(), match_size, opps),
        };
        digest = Pairings {
            paired: temp.paired,
            rejected: digest.rejected,
        };
    }
    digest
}

/// Takes a stream of players and creates pairings
fn process(
    plyrs: impl Iterator<Item=PlayerId>,
    match_size: usize,
    opps: &HashMap<PlayerId, HashSet<PlayerId>>,
) -> Pairings {
    let mut digest = Pairings {
        paired: Vec::new(),
        rejected: Vec::new(),
    };
    let mut queue = Vec::with_capacity(2 * match_size);
    for chunk in &plyrs.into_iter().chunks(match_size) {
        queue.extend(chunk);
        let (left, right, _) = Partitions::new(queue.iter().cloned(), match_size)
            .map(|(l, r)| {
                let count = count_opps(&l, opps) + count_opps(&r, opps);
                (l, r, count)
            })
            .min_by(|a, b| a.2.cmp(&b.2))
            .unwrap();
        digest.paired.push(left);
        queue.retain(|p| right.contains(p));
    }
    match queue.len() == match_size {
        true => {
            digest.paired.push(queue);
        }
        false => {
            digest.rejected = queue;
        }
    }
    digest
}

struct Partitions<T>
    where
        T: Clone,
{
    vals: Vec<T>,
    combos: Combinations<IntoIter<T>>,
}

impl<T> Iterator for Partitions<T>
    where
        T: Clone + PartialEq,
{
    type Item = (Vec<T>, Vec<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let combo = self.combos.next()?;
        let mut other = self.vals.clone();
        other.retain(|i| !combo.contains(i));
        Some((combo, other))
    }
}

impl<T> Partitions<T>
    where
        T: Clone,
{
    fn new(vals: impl Iterator<Item=T>, size: usize) -> Self {
        let vals: Vec<_> = vals.collect();
        let combos = vals.clone().into_iter().combinations(size);
        Self { vals, combos }
    }
}
