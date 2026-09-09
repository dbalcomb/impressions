//! A sparse region of memory.

mod cursor;
mod error;
mod segment;

use std::fmt::{self, Debug};
use std::iter::once;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::regions::uninitialized::Uninitialized;
use crate::memory::segmented::{Segmented, Segments};

pub use self::cursor::SegmentCursor;
pub use self::error::Error;
pub use self::segment::Segment;

/// A sparse region of memory composed of occupied and/or vacant segments.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Sparse<T>(Vec<Segment<T>>);

impl<T> Sparse<T> {
    /// Constructs a new sparse region with the given size.
    pub fn new(size: Size) -> Self {
        Self::vacant(Uninitialized::new(size))
    }

    /// Constructs a new sparse region from an uninitialized region.
    pub fn vacant(uninitialized: Uninitialized) -> Self {
        Self(vec![Segment::Vacant(uninitialized)])
    }

    /// Constructs a new sparse region with the given size value.
    pub fn from_size_value(size: u64) -> Result<Self, Error> {
        Ok(Self::new(Size::new(size)?))
    }

    /// Constructs a new sparse region from an iterator of segments.
    pub fn try_from_iterator(segments: impl IntoIterator<Item = Segment<T>>) -> Result<Self, Error>
    where
        T: Extent,
    {
        Self::try_from(segments.into_iter().collect::<Vec<Segment<T>>>())
    }
}

impl<T> Sparse<T>
where
    T: Extent,
{
    /// Inserts a region into a vacant space.
    pub fn insert(&mut self, address: Address, region: T) -> Result<(), Error> {
        let address_space = self.address_space();
        let region_address_space = address.to_space(region.size())?;

        if !self.address_space().includes(region_address_space) {
            return Err(Error::OutOfBounds(region_address_space, address_space));
        }

        let mut selected = self.segments().into_iter().select(region_address_space);

        let first = selected
            .next()
            .expect("a contained address space starts in a segment");

        if first.is_occupied() {
            return Err(Error::AlreadyOccupied(first.index()));
        }

        let mut last = first.clone();

        for segment in selected {
            if segment.is_occupied() {
                return Err(Error::AlreadyOccupied(segment.index()));
            }

            last = segment;
        }

        let before = first
            .address_space()
            .subtract(region_address_space)
            .before()
            .map(|address_space| Segment::vacant(Uninitialized::new(address_space.size())));

        let after = last
            .address_space()
            .subtract(region_address_space)
            .after()
            .map(|address_space| Segment::vacant(Uninitialized::new(address_space.size())));

        self.0.splice(
            first.index()..=last.index(),
            before
                .into_iter()
                .chain(once(Segment::occupied(region)))
                .chain(after),
        );

        Ok(())
    }
}

impl<T> Extent for Sparse<T>
where
    T: Extent,
{
    fn size(&self) -> Size {
        Size::try_sum(self.0.iter().map(Extent::size))
            .expect("sum of sizes does not exceed maximum size")
    }
}

impl<T> Segmented for Sparse<T>
where
    T: Extent,
{
    type Segment = Segment<T>;

    fn segments(&self) -> Segments<'_, Segment<T>> {
        Segments::new(&self.0)
    }
}

impl<T> Completion for Sparse<T>
where
    T: Completion,
{
    fn identified(&self) -> u64 {
        self.0.iter().map(Completion::identified).sum()
    }
}

impl<T> Debug for Sparse<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<T> From<Uninitialized> for Sparse<T>
where
    T: Extent,
{
    fn from(uninitialized: Uninitialized) -> Self {
        Self::vacant(uninitialized)
    }
}

impl<T> TryFrom<Vec<Segment<T>>> for Sparse<T>
where
    T: Extent,
{
    type Error = Error;

    fn try_from(segments: Vec<Segment<T>>) -> Result<Self, Self::Error> {
        Size::try_sum(segments.iter().map(Extent::size))?;

        Ok(Self(segments))
    }
}

impl<'de, T> Deserialize<'de> for Sparse<T>
where
    T: Deserialize<'de> + Extent,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from(<Vec<Segment<T>>>::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::{Address, AddressSpace};
    use crate::memory::extent::{Extent, Size};
    use crate::memory::regions::uninitialized::Uninitialized;

    use super::{Error, Segment, Sparse};

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Node(u64);

    impl Extent for Node {
        fn size(&self) -> Size {
            Size::new(self.0).unwrap()
        }
    }

    fn uninitialized(size: u64) -> Uninitialized {
        Uninitialized::new(Size::new(size).unwrap())
    }

    fn sparse(segments: impl IntoIterator<Item = Segment<Node>>) -> Sparse<Node> {
        Sparse::try_from_iterator(segments).unwrap()
    }

    #[test]
    fn insert_splits_single_vacant_segment() {
        let mut region = Sparse::new(Size::new(10).unwrap());

        region.insert(Address::new(3), Node(4)).unwrap();

        assert_eq!(
            region,
            sparse([
                Segment::vacant(uninitialized(3)),
                Segment::occupied(Node(4)),
                Segment::vacant(uninitialized(3)),
            ]),
        );
    }

    #[test]
    fn insert_at_segment_start_omits_empty_prefix() {
        let mut region = Sparse::new(Size::new(10).unwrap());

        region.insert(Address::new(0), Node(3)).unwrap();

        assert_eq!(
            region,
            sparse([
                Segment::occupied(Node(3)),
                Segment::vacant(uninitialized(7)),
            ]),
        );
    }

    #[test]
    fn insert_at_segment_end_omits_empty_suffix() {
        let mut region = Sparse::new(Size::new(10).unwrap());

        region.insert(Address::new(7), Node(3)).unwrap();

        assert_eq!(
            region,
            sparse([
                Segment::vacant(uninitialized(7)),
                Segment::occupied(Node(3)),
            ]),
        );
    }

    #[test]
    fn insert_replaces_entire_vacant_segment() {
        let mut region = Sparse::new(Size::new(10).unwrap());

        region.insert(Address::new(0), Node(10)).unwrap();

        assert_eq!(region, sparse([Segment::occupied(Node(10))]));
    }

    #[test]
    fn insert_replaces_multiple_vacant_segments() {
        let mut region = sparse([
            Segment::vacant(uninitialized(10)),
            Segment::vacant(uninitialized(10)),
            Segment::vacant(uninitialized(10)),
        ]);

        region.insert(Address::new(5), Node(20)).unwrap();

        assert_eq!(
            region,
            sparse([
                Segment::vacant(uninitialized(5)),
                Segment::occupied(Node(20)),
                Segment::vacant(uninitialized(5)),
            ]),
        );
    }

    #[test]
    fn insert_across_exact_segment_boundaries_omits_outer_empty_segments() {
        let mut region = sparse([
            Segment::vacant(uninitialized(10)),
            Segment::vacant(uninitialized(10)),
            Segment::vacant(uninitialized(10)),
        ]);

        region.insert(Address::new(10), Node(10)).unwrap();

        assert_eq!(
            region,
            sparse([
                Segment::vacant(uninitialized(10)),
                Segment::occupied(Node(10)),
                Segment::vacant(uninitialized(10)),
            ]),
        );
    }

    #[test]
    fn insert_rejects_range_that_overlaps_occupied_segment() {
        let mut region = Sparse::new(Size::new(10).unwrap());

        region.insert(Address::new(3), Node(4)).unwrap();

        let original = region.clone();

        assert_eq!(
            region.insert(Address::new(2), Node(3)),
            Err(Error::AlreadyOccupied(1)),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn insert_rejects_out_of_bounds_start() {
        let mut region = Sparse::new(Size::new(10).unwrap());
        let original = region.clone();

        assert_eq!(
            region.insert(Address::new(10), Node(1)),
            Err(Error::OutOfBounds(
                AddressSpace::new(Address::new(10), Address::new(10)).unwrap(),
                AddressSpace::new(Address::new(0), Address::new(9)).unwrap(),
            )),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn insert_rejects_arange_that_extends_past_region() {
        let mut region = Sparse::new(Size::new(10).unwrap());
        let original = region.clone();

        assert_eq!(
            region.insert(Address::new(8), Node(3)),
            Err(Error::OutOfBounds(
                AddressSpace::new(Address::new(8), Address::new(10)).unwrap(),
                AddressSpace::new(Address::new(0), Address::new(9)).unwrap(),
            ))
        );
        assert_eq!(region, original);
    }
}
