//! A sparse region of memory.

mod error;
mod segment;

use std::fmt::{self, Debug};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::regions::uninitialized::Uninitialized;
use crate::memory::segmented::{Segmented, Segments};
use crate::memory::{Extent, Slice};

pub use self::error::Error;
pub use self::segment::Segment;

/// A sparse region of memory composed of occupied and/or vacant segments.
#[derive(Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Sparse<T>(Vec<Segment<T>>);

impl<T> Sparse<T> {
    /// Constructs a new sparse region with the given size.
    pub fn new(size: u64) -> Result<Self, Error> {
        match size {
            0 => Ok(Self(Vec::new())),
            size => Ok(Self::vacant(Uninitialized::new(size)?)),
        }
    }

    /// Constructs a new sparse region from an uninitialized region.
    pub fn vacant(uninitialized: Uninitialized) -> Self {
        match uninitialized.size() {
            0 => Self(Vec::new()),
            _ => Self(vec![Segment::Vacant(uninitialized)]),
        }
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
            .checked_add(region_size)
            .filter(|&end| end <= total_size)
            .ok_or(Error::OutOfBounds(address, total_size))?;

        let mut selected = self.segments().into_iter().select(address, region.size());

        let first = if region.size() == 0 {
            self.get(address)
                .ok_or(Error::OutOfBounds(address, self.size()))?
        } else {
            selected
                .next()
                .ok_or(Error::OutOfBounds(address, self.size()))?
        };

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

        let before = (address > first.start_address())
            .then(|| {
                first_vacant.slice(
                    Address::new(0),
                    u64::from(address.value() - first.start_address().value()),
                )
            })
            .transpose()?;

        let after_offset = end - u64::from(last.start_address().value());
        let after = (after_offset < last.size())
            .then(|| {
                last_vacant.slice(
                    Address::new(after_offset as u32),
                    last.size() - after_offset,
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
    fn size(&self) -> u64 {
        self.0.iter().map(Extent::size).sum()
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

    fn try_from(mut segments: Vec<Segment<T>>) -> Result<Self, Self::Error> {
        let size: u64 = segments.iter().map(Extent::size).sum();

        segments.retain(|segment| segment.is_occupied() || segment.size() > 0);

        if let Some(segment) = segments.last()
            && segment.size() == 0
            && size == u32::MAX as u64 + 1
        {
            return Err(Error::UnaddressableSegment(segments.len() - 1));
        }

        if size > u32::MAX as u64 + 1 {
            return Err(Error::SizeTooLarge(size));
        }

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
    use crate::memory::Extent;
    use crate::memory::address::Address;
    use crate::memory::regions::uninitialized::Uninitialized;
    use crate::memory::segmented::Segmented;

    use super::{Error, Segment, Sparse};

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Node(u64);

    impl Extent for Node {
        fn size(&self) -> u64 {
            self.0
        }
    }

    fn uninitialized(size: u64) -> Uninitialized {
        Uninitialized::new(size).unwrap()
    }

    fn sparse(segments: impl IntoIterator<Item = Segment<Node>>) -> Sparse<Node> {
        Sparse::try_from_iterator(segments).unwrap()
    }

    #[test]
    fn insert_splits_single_vacant_segment() {
        let mut region = Sparse::new(10).unwrap();

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
        let mut region = Sparse::new(10).unwrap();

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
        let mut region = Sparse::new(10).unwrap();

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
        let mut region = Sparse::new(10).unwrap();

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
        let mut region = Sparse::new(10).unwrap();

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
        let mut region = Sparse::new(10).unwrap();
        let original = region.clone();

        assert_eq!(
            region.insert(Address::new(10), Node(1)),
            Err(Error::OutOfBounds(Address::new(10), 10)),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn insert_rejects_arange_that_extends_past_region() {
        let mut region = Sparse::new(10).unwrap();
        let original = region.clone();

        assert_eq!(
            region.insert(Address::new(8), Node(3)),
            Err(Error::OutOfBounds(Address::new(8), 10))
        );
        assert_eq!(region, original);
    }

    #[test]
    fn insert_allows_empty_region_at_addressable_offset() {
        let mut region = Sparse::new(10).unwrap();

        region.insert(Address::new(3), Node(0)).unwrap();

        assert_eq!(
            region,
            sparse([
                Segment::vacant(uninitialized(3)),
                Segment::occupied(Node(0)),
                Segment::vacant(uninitialized(7)),
            ]),
        );
    }

    #[test]
    fn segments_select_skips_empty_markers_at_range_boundaries() {
        let region = sparse([
            Segment::vacant(uninitialized(5)),
            Segment::occupied(Node(0)),
            Segment::vacant(uninitialized(5)),
            Segment::occupied(Node(0)),
            Segment::vacant(uninitialized(5)),
        ]);

        let start_indices = region
            .segments()
            .into_iter()
            .select(Address::new(5), 5)
            .map(|entry| entry.index())
            .collect::<Vec<_>>();

        assert_eq!(start_indices, [2]);

        let crossing_indices = region
            .segments()
            .into_iter()
            .select(Address::new(4), 2)
            .map(|entry| entry.index())
            .collect::<Vec<_>>();

        assert_eq!(crossing_indices, [0, 1, 2]);
    }
}
