use std::{cmp::Ordering, iter::Peekable};

/// This iterator takes two iterators and yields elements from them according to their ordering.
/// Ideally, this is done with two iterator that are already sorted. This will create an sequence
/// that is sorted without needing to allocate an intermediate buffer.
pub struct OrderedZip<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    left: Peekable<L>,
    right: Peekable<R>,
}

impl<L, R> OrderedZip<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
    L::Item: Ord,
{
    /// The constructor for the iterator.
    pub fn new(left: L, right: R) -> Self {
        Self {
            left: left.peekable(),
            right: right.peekable(),
        }
    }
}

impl<L, R> Iterator for OrderedZip<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
    L::Item: Ord,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // We peek at nothing on the left only when the iterator is empty, so drain the right iter
        let Some(left_peek) = self.left.peek() else {
            return self.right.next();
        };
        // And visa verse
        let Some(right_peek) = self.right.peek() else {
            return self.left.next();
        };
        match left_peek.cmp(right_peek) {
            Ordering::Less | Ordering::Equal => self.left.next(),
            Ordering::Greater => self.right.next(),
        }
    }
}

impl<L, R> ExactSizeIterator for OrderedZip<L, R>
where
    L: ExactSizeIterator,
    R: ExactSizeIterator<Item = L::Item>,
    L::Item: Ord,
{
    fn len(&self) -> usize {
        self.left.len() + self.right.len()
    }
}

pub struct OrderedFilter<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    left: Peekable<L>,
    right: Peekable<R>,
}

impl<L, R> OrderedFilter<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    /// The constructor for the iterator.
    pub fn new(left: L, right: R) -> Self {
        Self {
            left: left.peekable(),
            right: right.peekable(),
        }
    }
}

impl<L, R> Iterator for OrderedFilter<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
    L::Item: Ord,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let left_peek = self.left.peek()?;
            let right_peek = self.right.peek()?;
            match left_peek.cmp(right_peek) {
                // l = r, so return that value
                Ordering::Equal => {
                    drop(self.left.next());
                    return self.right.next();
                }
                // l < r, so drop r and continue
                Ordering::Less => drop(self.left.next()),
                // l > r, so drop r and continue
                Ordering::Greater => drop(self.right.next()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OrderedFilter, OrderedZip};

    fn get_odds(end: u32) -> impl Iterator<Item = u32> {
        (0..end).map(|i| 2 * i + 1)
    }

    fn get_evens(end: u32) -> impl Iterator<Item = u32> {
        (0..end).map(|i| 2 * i)
    }

    fn asssert_sorted<I, T>(mut iter: I)
    where
        I: Iterator<Item = T>,
        T: Ord,
    {
        let Some(mut item) = iter.next() else {
            return;
        };
        for next in iter {
            assert!(item < next);
            item = next;
        }
    }

    #[test]
    fn one_empty_iter() {
        let expected: Vec<_> = (0..10).collect();
        let actual: Vec<_> = OrderedZip::new(0..10, 0..0).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
        let actual: Vec<_> = OrderedZip::new(0..0, 0..10).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
    }

    #[test]
    fn interleave() {
        let mut expected: Vec<_> = get_odds(10).chain(get_evens(10)).collect();
        expected.sort();
        let actual: Vec<_> = OrderedZip::new(get_evens(10), get_odds(10)).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
        let actual: Vec<_> = OrderedZip::new(get_odds(10), get_evens(10)).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
    }

    #[test]
    fn one_longer_iterator() {
        let mut expected: Vec<_> = get_odds(10).chain(get_evens(20)).collect();
        expected.sort();
        let actual: Vec<_> = OrderedZip::new(get_evens(20), get_odds(10)).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
        let actual: Vec<_> = OrderedZip::new(get_odds(10), get_evens(20)).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
    }

    #[test]
    fn disjoint_iterators() {
        let expected: Vec<_> = (0..10).chain(100..110).collect();
        let actual: Vec<_> = OrderedZip::new(0..10, 100..110).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
        let actual: Vec<_> = OrderedZip::new(100..110, 0..10).collect();
        asssert_sorted(actual.iter());
        assert_eq!(actual, expected);
    }

    #[test]
    fn always_equal() {
        let expected: Vec<_> = (0..10).collect();
        let actual: Vec<_> = OrderedFilter::new(0..10, 0..10).collect();
        asssert_sorted(actual.iter());
        assert_eq!(expected, actual);
    }

    #[test]
    fn never_equal() {
        let expected = Vec::<u32>::new();
        let actual: Vec<_> = OrderedFilter::new(get_odds(10), get_evens(10)).collect();
        asssert_sorted(actual.iter());
        assert_eq!(expected, actual);
    }

    #[test]
    fn one_missing() {
        let expected = vec![0, 1, 3, 4, 5, 6, 7, 8, 9];
        let left = 0..10;
        let right = (0..2).chain(3..10);
        let actual: Vec<_> = OrderedFilter::new(left, right).collect();
        asssert_sorted(actual.iter());
        assert_eq!(expected, actual);
    }

    #[test]
    fn equal_once() {
        // First item matches
        let expected = vec![1];
        let left = get_odds(10);
        let right = std::iter::once(1).chain(get_evens(10));
        let actual: Vec<_> = OrderedFilter::new(left, right).collect();
        asssert_sorted(actual.iter());
        assert_eq!(expected, actual);

        // Last item matches
        let expected = vec![19];
        let left = get_odds(10);
        let right = get_evens(10).chain(std::iter::once(19));
        let actual: Vec<_> = OrderedFilter::new(left, right).collect();
        asssert_sorted(actual.iter());
        assert_eq!(expected, actual);
    }

    #[test]
    fn some_match() {
        // Right is a subset of left
        let expected: Vec<_> = get_evens(10).collect();
        let left = 0..=20;
        let right = get_evens(10);
        let actual: Vec<_> = OrderedFilter::new(left, right).collect();
        asssert_sorted(actual.iter());
        assert_eq!(expected, actual);

        // Right and left have an intersection but are not subsets
        let expected = vec![3, 4, 5];
        let actual: Vec<_> = OrderedFilter::new(0..=5, 3..10).collect();
        asssert_sorted(actual.iter());
        assert_eq!(expected, actual);
    }
}
