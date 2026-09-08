use std::iter::FusedIterator;
use std::range::RangeInclusive;

use super::AddressSpace;

/// A subtraction of two address spaces.
///
/// This represents the parts of the first address space that are not covered by
/// the second. This may result in zero, one, or two address spaces depending on
/// the overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subtraction {
    lhs: AddressSpace,
    rhs: AddressSpace,
}

impl Subtraction {
    /// Constructs a new subtraction of two address spaces.
    pub(super) const fn new(lhs: AddressSpace, rhs: AddressSpace) -> Self {
        Self { lhs, rhs }
    }
}

impl Subtraction {
    /// Computes the address space before the intersection of the two address
    /// spaces.
    pub const fn before(self) -> Option<AddressSpace> {
        let Some(intersection) = self.lhs.intersection(self.rhs) else {
            return None;
        };

        if intersection.first().value() == self.lhs.first().value() {
            return None;
        }

        let Some(last) = intersection.first().prev() else {
            return None;
        };

        Some(AddressSpace(RangeInclusive {
            start: self.lhs.first(),
            last,
        }))
    }

    /// Computes the address space after the intersection of the two address
    /// spaces.
    pub const fn after(self) -> Option<AddressSpace> {
        let Some(intersection) = self.lhs.intersection(self.rhs) else {
            return None;
        };

        if intersection.last().value() == self.lhs.last().value() {
            return None;
        }

        let Some(start) = intersection.last().next() else {
            return None;
        };

        Some(AddressSpace(RangeInclusive {
            start,
            last: self.lhs.last(),
        }))
    }
}

impl IntoIterator for Subtraction {
    type Item = AddressSpace;
    type IntoIter = SubtractionIter;

    fn into_iter(self) -> Self::IntoIter {
        if self.lhs.intersects(self.rhs) {
            SubtractionIter([self.before(), self.after()])
        } else {
            SubtractionIter([Some(self.lhs), None])
        }
    }
}

/// An iterator over the address spaces resulting from a subtraction.
#[derive(Clone, Debug)]
pub struct SubtractionIter([Option<AddressSpace>; 2]);

impl Iterator for SubtractionIter {
    type Item = AddressSpace;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(before) = self.0[0].take() {
            return Some(before);
        }

        if let Some(after) = self.0[1].take() {
            return Some(after);
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }
}

impl DoubleEndedIterator for SubtractionIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(after) = self.0[1].take() {
            return Some(after);
        }

        if let Some(before) = self.0[0].take() {
            return Some(before);
        }

        None
    }
}

impl ExactSizeIterator for SubtractionIter {
    fn len(&self) -> usize {
        self.0
            .iter()
            .filter(|address_space| address_space.is_some())
            .count()
    }
}

impl FusedIterator for SubtractionIter {}

#[cfg(test)]
mod tests {
    use crate::memory::address::{Address, AddressSpace};

    fn space(first: u32, last: u32) -> AddressSpace {
        AddressSpace::new(Address::new(first), Address::new(last)).unwrap()
    }

    fn subtract(lhs: (u32, u32), rhs: (u32, u32)) -> Vec<AddressSpace> {
        space(lhs.0, lhs.1)
            .subtract(space(rhs.0, rhs.1))
            .into_iter()
            .collect()
    }

    #[test]
    fn subtraction_preserves_lhs_when_rhs_is_before_it() {
        assert_eq!(subtract((10, 19), (0, 9)), [space(10, 19)]);
    }

    #[test]
    fn subtraction_preserves_lhs_when_rhs_is_after_it() {
        assert_eq!(subtract((10, 19), (20, 29)), [space(10, 19)]);
    }

    #[test]
    fn subtraction_removes_an_equal_address_space() {
        assert_eq!(subtract((10, 19), (10, 19)), []);
    }

    #[test]
    fn subtraction_removes_lhs_when_rhs_contains_it() {
        assert_eq!(subtract((10, 19), (0, 29)), []);
    }

    #[test]
    fn subtraction_splits_lhs_when_rhs_is_contained_within_it() {
        assert_eq!(subtract((10, 19), (13, 16)), [space(10, 12), space(17, 19)]);
    }

    #[test]
    fn subtraction_keeps_suffix_when_rhs_overlaps_lhs_start() {
        assert_eq!(subtract((10, 19), (0, 13)), [space(14, 19)]);
    }

    #[test]
    fn subtraction_keeps_prefix_when_rhs_overlaps_lhs_end() {
        assert_eq!(subtract((10, 19), (16, 29)), [space(10, 15)]);
    }

    #[test]
    fn subtraction_handles_single_address_boundaries() {
        assert_eq!(subtract((10, 19), (10, 10)), [space(11, 19)]);
        assert_eq!(subtract((10, 19), (19, 19)), [space(10, 18)]);
    }

    #[test]
    fn subtraction_handles_address_space_boundaries() {
        assert_eq!(subtract((0, u32::MAX), (0, 0)), [space(1, u32::MAX)]);
        assert_eq!(
            subtract((0, u32::MAX), (u32::MAX, u32::MAX)),
            [space(0, u32::MAX - 1)],
        );
    }
}
