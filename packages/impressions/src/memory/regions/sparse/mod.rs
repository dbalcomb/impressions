//! A sparse region of memory.

mod error;
mod segment;

use std::fmt::{self, Debug};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::Slice;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::regions::uninitialized::Uninitialized;
use crate::memory::segmented::{Segmented, Segments};

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
        let region_size = region.size();
        let total_size = self.size();
        let end = u64::from(address.value())
            .checked_add(region_size.get())
            .filter(|&end| end <= total_size.get())
            .ok_or(Error::OutOfBounds(address, total_size))?;

        let mut selected = self.segments().into_iter().select(address, region.size());

        let first = selected
            .next()
            .ok_or(Error::OutOfBounds(address, total_size))?;

        if first.is_occupied() {
            return Err(Error::AlreadyOccupied(first.index()));
        }

        let mut last = None;

        for entry in selected {
            if entry.is_occupied() {
                return Err(Error::AlreadyOccupied(entry.index()));
            }

            last = Some(entry);
        }

        let last = last.as_ref().unwrap_or(&first);

        let first_vacant = first.as_vacant().expect("occupied segments were rejected");
        let last_vacant = last.as_vacant().expect("occupied segments were rejected");

        let before = (address > first.address())
            .then(|| {
                first_vacant.slice(
                    Address::new(0),
                    Size::new(u64::from(address.value() - first.address().value()))
                        .expect("valid size"),
                )
            })
            .transpose()?;

        let after_offset = end - u64::from(last.address().value());
        let after = (after_offset < last.size().get())
            .then(|| {
                last_vacant.slice(
                    Address::new(after_offset as u32),
                    Size::new(last.size().get() - after_offset).expect("valid size"),
                )
            })
            .transpose()?;

        let replacement = before
            .into_iter()
            .map(Segment::vacant)
            .chain(std::iter::once(Segment::occupied(region)))
            .chain(after.into_iter().map(Segment::vacant));

        self.0.splice(first.index()..=last.index(), replacement);

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
    use crate::memory::address::Address;
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
            Err(Error::OutOfBounds(Address::new(10), Size::new(10).unwrap())),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn insert_rejects_arange_that_extends_past_region() {
        let mut region = Sparse::new(Size::new(10).unwrap());
        let original = region.clone();

        assert_eq!(
            region.insert(Address::new(8), Node(3)),
            Err(Error::OutOfBounds(Address::new(8), Size::new(10).unwrap()))
        );
        assert_eq!(region, original);
    }
}
