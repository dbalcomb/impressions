mod error;

use core::range::RangeInclusive;
use std::fmt::{self, Debug, Display};
use std::ops::{Bound, RangeBounds};

use super::Address;

pub use self::error::Error;

/// Represents an address space in memory.
///
/// # Implementation
///
/// This is internally represented by an inclusive range as it is possible for
/// the address space to refer to the end of the available memory. Using an
/// exclusive range would require an alternative representation to a [`u32`] or
/// [`Address`] for the end position.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AddressSpace(RangeInclusive<Address>);

impl AddressSpace {
    /// Constructs a new address space.
    ///
    /// # Errors
    ///
    /// This constructor errors if the the last address is smaller than the
    /// first as that would produce an invalid address space.
    pub const fn new(first: Address, last: Address) -> Result<Self, Error> {
        if last.value() < first.value() {
            return Err(Error::Invalid);
        }

        Ok(Self(RangeInclusive { start: first, last }))
    }

    /// Constructs a new address space with the given size.
    ///
    /// # Errors
    ///
    /// This constructor errors if the given size is zero or would exceed the
    /// maximum address space.
    pub const fn with_size(address: Address, size: u64) -> Result<Self, Error> {
        if size == 0 || size > u32::MAX as u64 + 1 {
            return Err(Error::Invalid);
        }

        match address.checked_add((size - 1) as u32) {
            Some(last) => Ok(Self(RangeInclusive {
                start: address,
                last,
            })),
            None => Err(Error::Invalid),
        }
    }

    /// Constructs a new address space from the given range.
    ///
    /// # Errors
    ///
    /// This constructor errors if the provided range is empty or reversed as
    /// that would produce an invalid address space.
    pub fn from_range(range: impl RangeBounds<Address>) -> Result<Self, Error> {
        let first = match range.start_bound() {
            Bound::Included(&first) => first,
            Bound::Excluded(&first) if first == Address::MAX => Err(Error::Invalid)?,
            Bound::Excluded(&first) => first + 1,
            Bound::Unbounded => Address::MIN,
        };

        let last = match range.end_bound() {
            Bound::Included(&last) => last,
            Bound::Excluded(&last) if last == Address::MIN => Err(Error::Invalid)?,
            Bound::Excluded(&last) => last - 1,
            Bound::Unbounded => Address::MAX,
        };

        Self::new(first, last)
    }
}

impl AddressSpace {
    /// Gets the first address of this address space.
    pub const fn first(&self) -> Address {
        self.0.start
    }

    /// Gets the last address of this address space.
    pub const fn last(&self) -> Address {
        self.0.last
    }

    /// Gets the previous address before this address space.
    pub const fn prev(&self) -> Option<Address> {
        self.first().checked_sub(1)
    }

    /// Gets the next address after this address space.
    pub const fn next(&self) -> Option<Address> {
        self.last().checked_add(1)
    }

    /// Gets the size of this address space.
    ///
    /// # Implementation
    ///
    /// The size is represented as a `u64` as the maximum size of an address
    /// space is `u32::MAX + 1`, which cannot be represented as a `u32`.
    pub const fn size(&self) -> u64 {
        (self.last().value() - self.first().value()) as u64 + 1
    }
}

impl AddressSpace {
    /// Gets the address at the given offset.
    pub const fn get_address_at(&self, offset: u32) -> Option<Address> {
        match self.first().checked_add(offset) {
            Some(address) => match self.contains(address) {
                true => Some(address),
                false => None,
            },
            None => None,
        }
    }

    /// Gets the offset at the given address.
    pub const fn get_offset_at(&self, address: Address) -> Option<u32> {
        if !self.contains(address) {
            return None;
        }

        Some(address.value() - self.first().value())
    }
}

impl AddressSpace {
    /// Checks whether the address space contains the given address.
    pub const fn contains(&self, address: Address) -> bool {
        address.value() >= self.first().value() && address.value() <= self.last().value()
    }

    /// Checks whether the address space includes another address space.
    pub const fn includes(&self, other: Self) -> bool {
        self.contains(other.first()) && self.contains(other.last())
    }

    /// Checks whether the address spaces intersect.
    pub const fn intersects(&self, other: Self) -> bool {
        self.intersection(other).is_some()
    }

    /// Computes the intersection of the address spaces.
    pub const fn intersection(&self, other: Self) -> Option<Self> {
        let lf = self.first().value();
        let rf = other.first().value();
        let first = if lf > rf { lf } else { rf };

        let ll = self.last().value();
        let rl = other.last().value();
        let last = if ll < rl { ll } else { rl };

        if first <= last {
            Some(Self(RangeInclusive {
                start: Address::new(first),
                last: Address::new(last),
            }))
        } else {
            None
        }
    }

    /// Computes the union of the address spaces.
    ///
    /// This method constructs the smallest address space that contains both
    /// input address spaces.
    pub const fn union(&self, other: Self) -> Self {
        let lf = self.first().value();
        let rf = other.first().value();
        let first = if lf < rf { lf } else { rf };

        let ll = self.last().value();
        let rl = other.last().value();
        let last = if ll > rl { ll } else { rl };

        Self(RangeInclusive {
            start: Address::new(first),
            last: Address::new(last),
        })
    }

    /// Checks whether the address space is adjacent before another.
    pub const fn is_adjacent_before(&self, other: Self) -> bool {
        match self.next() {
            Some(address) => address.value() == other.first().value(),
            None => false,
        }
    }

    /// Checks whether the address space is adjacent after another.
    pub const fn is_adjacent_after(&self, other: Self) -> bool {
        match other.next() {
            Some(address) => address.value() == self.first().value(),
            None => false,
        }
    }

    /// Checks whether the address space is adjacent to another.
    pub const fn is_adjacent_to(&self, other: Self) -> bool {
        self.is_adjacent_after(other) || self.is_adjacent_before(other)
    }
}

impl Default for AddressSpace {
    fn default() -> Self {
        Self(RangeInclusive::from(Address::MIN..=Address::MAX))
    }
}

impl Display for AddressSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.0.start, self.0.last)
    }
}

impl Debug for AddressSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.0.start, self.0.last)
    }
}

impl TryFrom<(Address, Address)> for AddressSpace {
    type Error = Error;

    fn try_from((first, last): (Address, Address)) -> Result<Self, Self::Error> {
        Self::new(first, last)
    }
}

impl TryFrom<(Address, u64)> for AddressSpace {
    type Error = Error;

    fn try_from((address, size): (Address, u64)) -> Result<Self, Self::Error> {
        Self::with_size(address, size)
    }
}

impl TryFrom<RangeInclusive<Address>> for AddressSpace {
    type Error = Error;

    fn try_from(range: RangeInclusive<Address>) -> Result<Self, Self::Error> {
        Self::from_range(range)
    }
}

impl TryFrom<std::ops::RangeInclusive<Address>> for AddressSpace {
    type Error = Error;

    fn try_from(range: std::ops::RangeInclusive<Address>) -> Result<Self, Self::Error> {
        Self::from_range(range)
    }
}

#[cfg(test)]
mod tests {
    use super::{Address, AddressSpace, Error};

    #[test]
    fn test_new() {
        assert!(AddressSpace::new(0.into(), 0.into()).is_ok());
        assert!(AddressSpace::new(0.into(), 79.into()).is_ok());
        assert!(AddressSpace::new(0.into(), u32::MAX.into()).is_ok());

        assert!(AddressSpace::new(10.into(), 10.into()).is_ok());
        assert!(AddressSpace::new(10.into(), 79.into()).is_ok());
        assert!(AddressSpace::new(10.into(), u32::MAX.into()).is_ok());

        assert_eq!(AddressSpace::new(79.into(), 0.into()), Err(Error::Invalid));
        assert_eq!(AddressSpace::new(79.into(), 10.into()), Err(Error::Invalid));
        assert_eq!(
            AddressSpace::new(u32::MAX.into(), 0.into()),
            Err(Error::Invalid)
        );
    }

    #[test]
    fn test_with_size() {
        let address = AddressSpace::with_size(0.into(), 1).unwrap();

        assert_eq!(address.first(), Address::new(0));
        assert_eq!(address.last(), Address::new(0));

        let address = AddressSpace::with_size(0.into(), 100).unwrap();

        assert_eq!(address.first(), Address::new(0));
        assert_eq!(address.last(), Address::new(99));

        let address = AddressSpace::with_size(10.into(), 100).unwrap();

        assert_eq!(address.first(), Address::new(10));
        assert_eq!(address.last(), Address::new(109));

        let address = AddressSpace::with_size(0.into(), u32::MAX as u64).unwrap();

        assert_eq!(address.first(), Address::new(0));
        assert_eq!(address.last(), Address::new(u32::MAX - 1));

        let address = AddressSpace::with_size(0.into(), u32::MAX as u64 + 1).unwrap();

        assert_eq!(address.first(), Address::new(0));
        assert_eq!(address.last(), Address::new(u32::MAX));

        assert_eq!(AddressSpace::with_size(10.into(), 0), Err(Error::Invalid));
        assert_eq!(
            AddressSpace::with_size(10.into(), u32::MAX as u64),
            Err(Error::Invalid)
        );
    }

    #[test]
    fn test_from_range() {
        assert_eq!(
            AddressSpace::from_range(..),
            AddressSpace::new(0.into(), u32::MAX.into())
        );

        assert_eq!(
            AddressSpace::from_range(Address::new(0)..),
            AddressSpace::new(0.into(), u32::MAX.into()),
        );
        assert_eq!(
            AddressSpace::from_range(Address::new(1)..),
            AddressSpace::new(1.into(), u32::MAX.into()),
        );
        assert_eq!(
            AddressSpace::from_range(Address::MAX..),
            AddressSpace::new(u32::MAX.into(), u32::MAX.into()),
        );

        assert_eq!(
            AddressSpace::from_range(..Address::new(0)),
            Err(Error::Invalid),
        );
        assert_eq!(
            AddressSpace::from_range(..Address::new(1)),
            AddressSpace::new(0.into(), 0.into()),
        );
        assert_eq!(
            AddressSpace::from_range(..Address::MAX),
            AddressSpace::new(0.into(), Address::new(u32::MAX - 1)),
        );

        assert_eq!(
            AddressSpace::from_range(..=Address::new(0)),
            AddressSpace::new(0.into(), 0.into()),
        );
        assert_eq!(
            AddressSpace::from_range(..=Address::new(1)),
            AddressSpace::new(0.into(), 1.into()),
        );
        assert_eq!(
            AddressSpace::from_range(..=Address::MAX),
            AddressSpace::new(0.into(), u32::MAX.into()),
        );

        assert_eq!(
            AddressSpace::from_range(Address::new(0)..Address::new(0)),
            Err(Error::Invalid),
        );
        assert_eq!(
            AddressSpace::from_range(Address::new(0)..Address::new(1)),
            AddressSpace::new(0.into(), 0.into()),
        );
        assert_eq!(
            AddressSpace::from_range(Address::new(1)..Address::MAX),
            AddressSpace::new(1.into(), Address::new(u32::MAX - 1)),
        );

        assert_eq!(
            AddressSpace::from_range(Address::new(0)..=Address::new(0)),
            AddressSpace::new(0.into(), 0.into()),
        );
        assert_eq!(
            AddressSpace::from_range(Address::new(0)..=Address::new(1)),
            AddressSpace::new(0.into(), 1.into()),
        );
        assert_eq!(
            AddressSpace::from_range(Address::new(1)..=Address::MAX),
            AddressSpace::new(1.into(), u32::MAX.into()),
        );
    }

    #[test]
    fn text_prev() {
        assert_eq!(AddressSpace::new(0.into(), 79.into()).unwrap().prev(), None);
        assert_eq!(
            AddressSpace::new(10.into(), 79.into()).unwrap().prev(),
            Some(Address::new(9))
        );
    }

    #[test]
    fn test_next() {
        assert_eq!(
            AddressSpace::new(10.into(), u32::MAX.into())
                .unwrap()
                .next(),
            None
        );
        assert_eq!(
            AddressSpace::new(10.into(), 79.into()).unwrap().next(),
            Some(Address::new(80))
        );
    }

    #[test]
    fn test_size() {
        assert_eq!(AddressSpace::new(10.into(), 79.into()).unwrap().size(), 70);
        assert_eq!(
            AddressSpace::new(0.into(), u32::MAX.into()).unwrap().size(),
            (u32::MAX as u64) + 1
        );
    }

    #[test]
    fn test_relation_self() {
        let a = AddressSpace::new(10.into(), 79.into()).unwrap();

        assert!(a.includes(a));

        assert!(a.contains(a.first()));
        assert!(a.contains(a.last()));
        assert!(!a.contains(a.prev().unwrap()));
        assert!(!a.contains(a.next().unwrap()));

        assert!(a.intersects(a));
        assert_eq!(a.intersection(a), Some(a));

        assert!(!a.is_adjacent_before(a));
        assert!(!a.is_adjacent_after(a));
        assert!(!a.is_adjacent_to(a));

        assert_eq!(a.union(a), a);
    }

    #[test]
    fn test_relation_includes() {
        let a = AddressSpace::new(10.into(), 79.into()).unwrap();
        let b = AddressSpace::new(1.into(), 99.into()).unwrap();

        assert!(!a.includes(b));
        assert!(b.includes(a));

        assert!(!a.contains(b.first()));
        assert!(!a.contains(b.last()));
        assert!(!a.contains(b.prev().unwrap()));
        assert!(!a.contains(b.next().unwrap()));
        assert!(b.contains(a.first()));
        assert!(b.contains(a.last()));
        assert!(b.contains(a.prev().unwrap()));
        assert!(b.contains(a.next().unwrap()));

        assert!(a.intersects(b));
        assert_eq!(a.intersection(b), Some(a));
        assert!(b.intersects(a));
        assert_eq!(b.intersection(a), Some(a));

        assert!(!a.is_adjacent_before(b));
        assert!(!a.is_adjacent_after(b));
        assert!(!a.is_adjacent_to(b));
        assert!(!b.is_adjacent_before(a));
        assert!(!b.is_adjacent_after(a));
        assert!(!b.is_adjacent_to(a));

        assert_eq!(a.union(b), b);
        assert_eq!(b.union(a), b);
    }

    #[test]
    fn test_relation_disjoint() {
        let a = AddressSpace::new(10.into(), 79.into()).unwrap();
        let b = AddressSpace::new(5.into(), 8.into()).unwrap();

        assert!(!a.includes(b));
        assert!(!b.includes(a));

        assert!(!a.contains(b.first()));
        assert!(!a.contains(b.last()));
        assert!(!a.contains(b.prev().unwrap()));
        assert!(!a.contains(b.next().unwrap()));
        assert!(!b.contains(a.first()));
        assert!(!b.contains(a.last()));
        assert!(!b.contains(a.prev().unwrap()));
        assert!(!b.contains(a.next().unwrap()));

        assert!(!a.intersects(b));
        assert_eq!(a.intersection(b), None);
        assert!(!b.intersects(a));
        assert_eq!(b.intersection(a), None);

        assert!(!a.is_adjacent_before(b));
        assert!(!a.is_adjacent_after(b));
        assert!(!a.is_adjacent_to(b));
        assert!(!b.is_adjacent_before(a));
        assert!(!b.is_adjacent_after(a));
        assert!(!b.is_adjacent_to(a));

        assert_eq!(a.union(b), AddressSpace::new(5.into(), 79.into()).unwrap());
        assert_eq!(b.union(a), AddressSpace::new(5.into(), 79.into()).unwrap());
    }

    #[test]
    fn test_relation_adjacent() {
        let a = AddressSpace::new(10.into(), 79.into()).unwrap();
        let b = AddressSpace::new(3.into(), 9.into()).unwrap();

        assert!(!a.includes(b));
        assert!(!b.includes(a));

        assert!(!a.contains(b.first()));
        assert!(!a.contains(b.last()));
        assert!(!a.contains(b.prev().unwrap()));
        assert!(a.contains(b.next().unwrap()));
        assert!(!b.contains(a.first()));
        assert!(!b.contains(a.last()));
        assert!(b.contains(a.prev().unwrap()));
        assert!(!b.contains(a.next().unwrap()));

        assert!(!a.intersects(b));
        assert_eq!(a.intersection(b), None);
        assert!(!b.intersects(a));
        assert_eq!(b.intersection(a), None);

        assert!(!a.is_adjacent_before(b));
        assert!(a.is_adjacent_after(b));
        assert!(a.is_adjacent_to(b));
        assert!(b.is_adjacent_before(a));
        assert!(!b.is_adjacent_after(a));
        assert!(b.is_adjacent_to(a));

        assert_eq!(a.union(b), AddressSpace::new(3.into(), 79.into()).unwrap());
        assert_eq!(b.union(a), AddressSpace::new(3.into(), 79.into()).unwrap());
    }

    #[test]
    fn test_relation_overlaps() {
        let a = AddressSpace::new(10.into(), 79.into()).unwrap();
        let b = AddressSpace::new(5.into(), 10.into()).unwrap();

        assert!(!a.includes(b));
        assert!(!b.includes(a));

        assert!(!a.contains(b.first()));
        assert!(a.contains(b.last()));
        assert!(!a.contains(b.prev().unwrap()));
        assert!(a.contains(b.next().unwrap()));
        assert!(b.contains(a.first()));
        assert!(!b.contains(a.last()));
        assert!(b.contains(a.prev().unwrap()));
        assert!(!b.contains(a.next().unwrap()));

        assert!(a.intersects(b));
        assert_eq!(
            a.intersection(b),
            Some(AddressSpace::new(10.into(), 10.into()).unwrap())
        );
        assert!(b.intersects(a));
        assert_eq!(
            b.intersection(a),
            Some(AddressSpace::new(10.into(), 10.into()).unwrap())
        );

        assert!(!a.is_adjacent_before(b));
        assert!(!a.is_adjacent_after(b));
        assert!(!a.is_adjacent_to(b));
        assert!(!b.is_adjacent_before(a));
        assert!(!b.is_adjacent_after(a));
        assert!(!b.is_adjacent_to(a));

        assert_eq!(a.union(b), AddressSpace::new(5.into(), 79.into()).unwrap());
        assert_eq!(b.union(a), AddressSpace::new(5.into(), 79.into()).unwrap());
    }
}
