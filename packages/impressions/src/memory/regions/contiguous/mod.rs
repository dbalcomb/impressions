//! A contiguous region of memory.

mod error;
mod segment;

use std::fmt::{self, Debug};
use std::iter::once;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::regions::unidentified::Unidentified;
use crate::memory::segmented::{Segmented, Segments};
use crate::memory::slice::Slice;

pub use self::error::Error;
pub use self::segment::Segment;

/// A contiguous region of memory composed of identified and/or unidentified
/// segments.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Contiguous<T>(Vec<Segment<T>>);

impl<T> Contiguous<T> {
    /// Constructs a new contiguous region from an unidentified region.
    pub fn unidentified(unidentified: Unidentified) -> Self {
        Self(vec![Segment::Unidentified(unidentified)])
    }

    /// Constructs a new contiguous region from an iterator of segments.
    pub fn try_from_iterator(segments: impl IntoIterator<Item = Segment<T>>) -> Result<Self, Error>
    where
        T: Extent,
    {
        Self::try_from(segments.into_iter().collect::<Vec<Segment<T>>>())
    }
}

impl<T> Contiguous<T>
where
    T: Extent,
{
    /// Identifies the segment at the given address as the provided region.
    pub fn identify(&mut self, address: Address, region: T) -> Result<(), Error> {
        let address_space = self.address_space();
        let region_address_space = address.to_space(region.size())?;

        if !address_space.includes(region_address_space) {
            return Err(Error::OutOfBounds(region_address_space, address_space));
        }

        let mut selected = self.segments().into_iter().select(region_address_space);

        let first = selected
            .next()
            .expect("a contained address space starts in a segment");

        if first.is_identified() {
            return Err(Error::AlreadyIdentified(first.index()));
        }

        let mut last = first.clone();

        for segment in selected {
            if segment.is_identified() {
                return Err(Error::AlreadyIdentified(segment.index()));
            }

            last = segment;
        }

        let first_unidentified = first
            .as_unidentified()
            .expect("identified segments were rejected");

        let last_unidentified = last
            .as_unidentified()
            .expect("identified segments were rejected");

        let first_address_space = first.address_space();
        let before = first_address_space
            .subtract(region_address_space)
            .before()
            .map(|address_space| {
                first_unidentified
                    .slice(address_space.size().to_address_space())
                    .map(Segment::unidentified)
            })
            .transpose()?;

        let last_address_space = last.address_space();
        let after = last_address_space
            .subtract(region_address_space)
            .after()
            .map(|address_space| {
                let offset = last_address_space
                    .get_offset_at(address_space.first())
                    .expect("subtraction result is within the final segment");

                let local_address_space = Address::new(offset)
                    .to_space(address_space.size())
                    .expect("a subspace of a valid address space is valid");

                last_unidentified
                    .slice(local_address_space)
                    .map(Segment::unidentified)
            })
            .transpose()?;

        self.0.splice(
            first.index()..=last.index(),
            before
                .into_iter()
                .chain(once(Segment::identified(region)))
                .chain(after),
        );

        Ok(())
    }
}

impl<T> Extent for Contiguous<T>
where
    T: Extent,
{
    fn size(&self) -> Size {
        Size::try_sum(self.0.iter().map(Extent::size))
            .expect("sum of sizes does not exceed maximum size")
    }
}

impl<T> Segmented for Contiguous<T>
where
    T: Extent,
{
    type Segment = Segment<T>;

    fn segments(&self) -> Segments<'_, Segment<T>> {
        Segments::new(&self.0)
    }
}

impl<T> Completion for Contiguous<T>
where
    T: Completion,
{
    fn identified(&self) -> u64 {
        self.0.iter().map(Completion::identified).sum()
    }
}

impl<T> Debug for Contiguous<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<T> From<Unidentified> for Contiguous<T>
where
    T: Extent,
{
    fn from(unidentified: Unidentified) -> Self {
        Self::unidentified(unidentified)
    }
}

impl<T> TryFrom<Vec<Segment<T>>> for Contiguous<T>
where
    T: Extent,
{
    type Error = Error;

    fn try_from(segments: Vec<Segment<T>>) -> Result<Self, Self::Error> {
        Size::try_sum(segments.iter().map(Extent::size))?;

        Ok(Self(segments))
    }
}

impl<'de, T> Deserialize<'de> for Contiguous<T>
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
    use bytes::Bytes;

    use crate::memory::address::{Address, AddressSpace};
    use crate::memory::extent::{Extent, Size};
    use crate::memory::regions::unidentified::Unidentified;

    use super::{Contiguous, Error, Segment};

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Node(u64);

    impl Extent for Node {
        fn size(&self) -> Size {
            Size::new(self.0).unwrap()
        }
    }

    fn initialized(bytes: &'static [u8]) -> Unidentified {
        Unidentified::try_from_initialized_bytes(Bytes::from_static(bytes)).unwrap()
    }

    fn uninitialized(size: u64) -> Unidentified {
        Unidentified::try_from_uninitialized_size(size).unwrap()
    }

    fn both(bytes: &'static [u8], size: u64) -> Unidentified {
        Unidentified::try_from_initialized_bytes(Bytes::from_static(bytes))
            .unwrap()
            .with_uninitialized_size(size)
            .unwrap()
    }

    fn contiguous(segments: impl IntoIterator<Item = Segment<Node>>) -> Contiguous<Node> {
        Contiguous::try_from_iterator(segments).unwrap()
    }

    #[test]
    fn identify_splits_single_unidentified_segment() {
        let mut region = Contiguous::unidentified(initialized(b"0123456789"));

        region.identify(Address::new(3), Node(4)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(initialized(b"012")),
                Segment::identified(Node(4)),
                Segment::unidentified(initialized(b"789")),
            ]),
        );
    }

    #[test]
    fn identify_at_segment_start_omits_empty_prefix() {
        let mut region = Contiguous::unidentified(initialized(b"0123456789"));

        region.identify(Address::new(0), Node(3)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::identified(Node(3)),
                Segment::unidentified(initialized(b"3456789")),
            ]),
        );
    }

    #[test]
    fn identify_at_segment_end_omits_empty_suffix() {
        let mut region = Contiguous::unidentified(initialized(b"0123456789"));

        region.identify(Address::new(7), Node(3)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(initialized(b"0123456")),
                Segment::identified(Node(3)),
            ]),
        );
    }

    #[test]
    fn identify_replaces_entire_unidentified_segment() {
        let mut region = Contiguous::unidentified(initialized(b"0123456789"));

        region.identify(Address::new(0), Node(10)).unwrap();

        assert_eq!(region, contiguous([Segment::identified(Node(10))]));
    }

    #[test]
    fn identify_replaces_multiple_unidentified_segments() {
        let mut region = contiguous([
            Segment::unidentified(initialized(b"aaaaaaaaaa")),
            Segment::unidentified(initialized(b"bbbbbbbbbb")),
            Segment::unidentified(initialized(b"cccccccccc")),
        ]);

        region.identify(Address::new(5), Node(20)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(initialized(b"aaaaa")),
                Segment::identified(Node(20)),
                Segment::unidentified(initialized(b"ccccc")),
            ]),
        );
    }

    #[test]
    fn identify_across_exact_segment_boundaries_omits_outer_empty_segments() {
        let mut region = contiguous([
            Segment::unidentified(initialized(b"aaaaaaaaaa")),
            Segment::unidentified(initialized(b"bbbbbbbbbb")),
            Segment::unidentified(initialized(b"cccccccccc")),
        ]);

        region.identify(Address::new(10), Node(10)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(initialized(b"aaaaaaaaaa")),
                Segment::identified(Node(10)),
                Segment::unidentified(initialized(b"cccccccccc")),
            ]),
        );
    }

    #[test]
    fn identify_rejects_range_that_overlaps_identified_segment() {
        let mut region = Contiguous::unidentified(initialized(b"0123456789"));

        region.identify(Address::new(3), Node(4)).unwrap();

        let original = region.clone();

        assert_eq!(
            region.identify(Address::new(2), Node(3)),
            Err(Error::AlreadyIdentified(1)),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn identify_rejects_out_of_bounds_start() {
        let mut region = Contiguous::unidentified(initialized(b"0123456789"));
        let original = region.clone();

        assert_eq!(
            region.identify(Address::new(10), Node(1)),
            Err(Error::OutOfBounds(
                AddressSpace::new(Address::new(10), Address::new(10)).unwrap(),
                AddressSpace::new(Address::new(0), Address::new(9)).unwrap(),
            )),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn identify_rejects_range_that_extends_past_region() {
        let mut region = Contiguous::unidentified(initialized(b"0123456789"));
        let original = region.clone();

        assert_eq!(
            region.identify(Address::new(8), Node(3)),
            Err(Error::OutOfBounds(
                AddressSpace::new(Address::new(8), Address::new(10)).unwrap(),
                AddressSpace::new(Address::new(0), Address::new(9)).unwrap(),
            ))
        );
        assert_eq!(region, original);
    }

    #[test]
    fn identify_preserves_uninitialized_memory() {
        let mut region = Contiguous::unidentified(both(b"abcd", 6));

        region.identify(Address::new(2), Node(5)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(initialized(b"ab")),
                Segment::identified(Node(5)),
                Segment::unidentified(uninitialized(3)),
            ]),
        );
    }

    #[test]
    fn identify_can_start_in_uninitialized_memory() {
        let mut region = Contiguous::unidentified(both(b"abcd", 6));

        region.identify(Address::new(6), Node(2)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(both(b"abcd", 2)),
                Segment::identified(Node(2)),
                Segment::unidentified(uninitialized(2)),
            ]),
        );
    }
}
