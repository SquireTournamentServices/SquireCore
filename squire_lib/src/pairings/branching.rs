use std::collections::{HashMap, HashSet};

use crate::{identifiers::PlayerId, pairings::Pairings};

struct PairingTree {
    id: PlayerId,
    branches: Vec<PairingTree>,
}

#[allow(unused)]
/// The branching pairings impl
pub fn branching_pairings(
    mut plyrs: Vec<PlayerId>,
    opps: &HashMap<PlayerId, HashSet<PlayerId>>,
    match_size: usize,
    _: u64,
) -> Pairings {
    let mut digest = Pairings {
        paired: Vec::with_capacity(plyrs.len() / match_size + 1),
        rejected: Vec::new(),
    };
    let mut is_paired: HashSet<PlayerId> = HashSet::with_capacity(plyrs.len());
    let empty = HashSet::new();
    while is_paired.len() != plyrs.len() {
        let mut iter = plyrs.iter().filter(|p| !is_paired.contains(p)).cloned();
        // This unwrap is safe as iter will always have its first item since is_paired is shorter
        // than plyrs
        let mut tree = PairingTree::new(iter.next().unwrap());
        let mut pairing = None;
        for plyr in iter {
            let opp = opps.get(&plyr).unwrap_or(&empty);
            if opp.contains(&tree.id) {
                continue;
            }
            tree.insert(plyr, opp);
            pairing = tree.cut(match_size);
            if pairing.is_some() {
                break;
            }
        }
        is_paired.insert(tree.id);
        match pairing {
            None => {
                digest.rejected.push(tree.id);
            }
            Some(pair) => {
                is_paired.extend(pair.iter().cloned());
                digest.paired.push(pair.into_iter().rev().collect());
            }
        }
    }
    digest
}

impl PairingTree {
    /// Creates a new tree
    fn new(id: PlayerId) -> Self {
        Self {
            id,
            branches: Vec::with_capacity(2),
        }
    }

    /// For each branch, checks to see if the id and the root of each branch are valid opponents.
    /// If there is any branch that isn't valid, the id is also inserted as a new branch.
    ///
    /// Note: `opps` is the set of past opponents of `id`.
    fn insert(&mut self, id: PlayerId, opps: &HashSet<PlayerId>) {
        let mut insert = self.branches.is_empty();
        for branch in self.branches.iter_mut() {
            if !opps.contains(&branch.id) {
                branch.insert(id, opps);
            } else {
                insert = true;
            }
        }
        if insert {
            self.branches.push(Self::new(id));
        }
    }

    /// Traverses the tree to find a complete pairing. Each branch tranverses its branches in the
    /// order that they were inserted
    fn cut(&mut self, size: usize) -> Option<Vec<PlayerId>> {
        if size == 1 {
            Some(vec![self.id])
        } else {
            let mut digest = self
                .branches
                .iter_mut()
                .find_map(|branch| branch.cut(size - 1));
            if let Some(pairing) = digest.as_mut() {
                pairing.push(self.id);
            }
            digest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{branching_pairings, PairingTree};
    use crate::identifiers::PlayerId;
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    #[test]
    fn simple_tree() {
        let ids = vec![
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
        ];

        let opps: HashMap<PlayerId, HashSet<PlayerId>> =
            ids.iter().map(|id| (*id, HashSet::new())).collect();
        let mut tree = PairingTree::new(ids[0]);
        assert!(tree.branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[1];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches.len(), 1);
        assert!(tree.branches[0].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[2];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches.len(), 1);
        assert!(tree.branches[0].branches[0].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[3];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches[0].branches.len(), 1);
        assert!(tree.branches[0].branches[0].branches[0].branches.is_empty());
        let expected_pairing: Vec<_> = ids.iter().cloned().rev().collect();
        assert_eq!(tree.cut(4).unwrap(), expected_pairing);

        // Use the alg to do the same thing
        let expected_pairing: Vec<_> = ids.iter().cloned().collect();
        let pairings = branching_pairings(ids, &opps, 4, 0);
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0], expected_pairing);
        assert!(pairings.rejected.is_empty());
    }

    #[test]
    fn single_branch() {
        let ids = vec![
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
        ];

        let mut opps: HashMap<PlayerId, HashSet<PlayerId>> =
            ids.iter().map(|id| (*id, HashSet::new())).collect();
        let c = ids[2];
        let d = ids[3];
        opps.get_mut(&c).unwrap().insert(d);
        opps.get_mut(&d).unwrap().insert(c);

        let mut tree = PairingTree::new(ids[0]);
        assert!(tree.branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[1];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches.len(), 1);
        assert!(tree.branches[0].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[2];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches.len(), 1);
        assert!(tree.branches[0].branches[0].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[3];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches.len(), 2);
        assert!(tree.branches[0].branches[0].branches.is_empty());
        assert!(tree.branches[0].branches[1].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[4];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches.len(), 2);
        assert_eq!(tree.branches[0].branches[0].branches.len(), 1);
        assert_eq!(tree.branches[0].branches[1].branches.len(), 1);
        let expected_pairing = vec![ids[4], ids[2], ids[1], ids[0]];
        assert_eq!(tree.cut(4).unwrap(), expected_pairing);

        // Use the alg to do the same thing
        let expected_pairing = vec![ids[0], ids[1], ids[2], ids[4]];
        let pairings = branching_pairings(ids, &opps, 4, 0);
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0], expected_pairing);
        assert_eq!(pairings.rejected.len(), 1);
    }

    #[test]
    fn triple_branch() {
        let ids = vec![
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
            PlayerId::new(Uuid::new_v4()),
        ];

        let mut opps: HashMap<PlayerId, HashSet<PlayerId>> =
            ids.iter().map(|id| (*id, HashSet::new())).collect();
        let c = ids[2];
        let d = ids[3];
        let e = ids[4];
        opps.get_mut(&c).unwrap().insert(d);
        opps.get_mut(&c).unwrap().insert(e);
        opps.get_mut(&e).unwrap().insert(c);
        opps.get_mut(&d).unwrap().insert(c);

        let mut tree = PairingTree::new(ids[0]);
        assert!(tree.branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[1];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches.len(), 1);
        assert!(tree.branches[0].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[2];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches.len(), 1);
        assert!(tree.branches[0].branches[0].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[3];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches.len(), 2);
        assert!(tree.branches[0].branches[0].branches.is_empty());
        assert!(tree.branches[0].branches[1].branches.is_empty());
        assert!(tree.cut(4).is_none());

        let id = ids[4];
        tree.insert(id, opps.get(&id).unwrap());
        assert_eq!(tree.branches[0].branches[1].branches.len(), 1);
        assert!(tree.branches[0].branches[0].branches.is_empty());
        assert_eq!(tree.branches[0].branches.len(), 3);
        assert!(tree.branches[0].branches[2].branches.is_empty());
        let expected_pairing = vec![ids[4], ids[3], ids[1], ids[0]];
        assert_eq!(tree.cut(4).unwrap(), expected_pairing);

        // Use the alg to do the same thing
        let expected_pairing = vec![ids[0], ids[1], ids[3], ids[4]];
        let pairings = branching_pairings(ids.clone(), &opps, 4, 0);
        assert_eq!(pairings.paired.len(), 1);
        assert_eq!(pairings.paired[0], expected_pairing);
        assert_eq!(pairings.rejected.len(), 1);
    }
}
