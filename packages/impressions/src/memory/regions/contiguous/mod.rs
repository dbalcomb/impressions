//! A contiguous region of memory.

mod entry;
mod error;
mod iter;
mod segment;

use std::fmt::{self, Debug};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::regions::unidentified::Unidentified;
use crate::memory::{Extent, Slice};

pub use self::entry::Entry;
pub use self::error::Error;
pub use self::iter::Segments;
pub use self::segment::Segment;

/// A contiguous region of memory composed of identified and/or unidentified
/// segments.
#[derive(Clone, Default, PartialEq, Eq, Serialize)]
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
    /// Identifies the segment at the given offset as the provided region.
    pub fn identify(&mut self, offset: u32, region: T) -> Result<(), Error> {
        let region_size = region.size();
        let total_size = self.size();
        let end = u64::from(offset)
            .checked_add(region_size)
            .filter(|&end| end <= total_size)
            .ok_or(Error::OutOfBounds(offset, total_size))?;

        if end > self.size() {
            return Err(Error::OutOfBounds(offset, self.size()));
        }

        let mut selected = self.segments().select(offset, region.size());

        let first = if region.size() == 0 {
            self.get(offset)
                .ok_or(Error::OutOfBounds(offset, self.size()))?
        } else {
            selected
                .next()
                .ok_or(Error::OutOfBounds(offset, self.size()))?
        };

        if first.is_identified() {
            return Err(Error::AlreadyIdentified(first.segment_index()));
        }

        let mut last = None;

        for entry in selected {
            if entry.is_identified() {
                return Err(Error::AlreadyIdentified(entry.segment_index()));
            }

            last = Some(entry);
        }

        let last = last.as_ref().unwrap_or(&first);

        let first_unidentified = first
            .as_unidentified()
            .expect("identified segments were rejected");

        let last_unidentified = last
            .as_unidentified()
            .expect("identified segments were rejected");

        let before = (offset > first.segment_offset())
            .then(|| first_unidentified.slice(0, u64::from(offset - first.segment_offset())))
            .transpose()?;

        let after_offset = end - u64::from(last.segment_offset());
        let after = (after_offset < last.size())
            .then(|| last_unidentified.slice(after_offset as u32, last.size() - after_offset))
            .transpose()?;

        let replacement = before
            .into_iter()
            .map(Segment::unidentified)
            .chain(std::iter::once(Segment::identified(region)))
            .chain(after.into_iter().map(Segment::unidentified));

        self.0
            .splice(first.segment_index()..=last.segment_index(), replacement);

        Ok(())
    }

    /// Gets the segment at the given offset.
    pub fn get(&self, offset: u32) -> Option<Entry<'_, T>> {
        self.segments().get(offset)
    }

    /// Gets an iterator over the segments.
    pub fn segments(&self) -> Segments<'_, T> {
        Segments::new(self)
    }
}

impl<T> Extent for Contiguous<T>
where
    T: Extent,
{
    fn size(&self) -> u64 {
        self.0.iter().map(Extent::size).sum()
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
        let size: u64 = segments.iter().map(Extent::size).sum();

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

    use crate::memory::Extent;
    use crate::memory::regions::unidentified::Unidentified;

    use super::{Contiguous, Error, Segment};

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Node(u64);

    impl Extent for Node {
        fn size(&self) -> u64 {
            self.0
        }
    }

    fn unidentified(bytes: &'static [u8], uninitialized: u64) -> Unidentified {
        Unidentified::new(Bytes::from_static(bytes), uninitialized).unwrap()
    }

    fn uninitialized(size: u64) -> Unidentified {
        Unidentified::new(Bytes::new(), size).unwrap()
    }

    fn contiguous(segments: impl IntoIterator<Item = Segment<Node>>) -> Contiguous<Node> {
        Contiguous::try_from_iterator(segments).unwrap()
    }

    #[test]
    fn identify_splits_single_unidentified_segment() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));

        region.identify(3, Node(4)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(unidentified(b"012", 0)),
                Segment::identified(Node(4)),
                Segment::unidentified(unidentified(b"789", 0)),
            ]),
        );
    }

    #[test]
    fn identify_at_segment_start_omits_empty_prefix() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));

        region.identify(0, Node(3)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::identified(Node(3)),
                Segment::unidentified(unidentified(b"3456789", 0)),
            ]),
        );
    }

    #[test]
    fn identify_at_segment_end_omits_empty_suffix() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));

        region.identify(7, Node(3)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(unidentified(b"0123456", 0)),
                Segment::identified(Node(3)),
            ]),
        );
    }

    #[test]
    fn identify_replaces_entire_unidentified_segment() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));

        region.identify(0, Node(10)).unwrap();

        assert_eq!(region, contiguous([Segment::identified(Node(10))]));
    }

    #[test]
    fn identify_replaces_multiple_unidentified_segments() {
        let mut region = contiguous([
            Segment::unidentified(unidentified(b"aaaaaaaaaa", 0)),
            Segment::unidentified(unidentified(b"bbbbbbbbbb", 0)),
            Segment::unidentified(unidentified(b"cccccccccc", 0)),
        ]);

        region.identify(5, Node(20)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(unidentified(b"aaaaa", 0)),
                Segment::identified(Node(20)),
                Segment::unidentified(unidentified(b"ccccc", 0)),
            ]),
        );
    }

    #[test]
    fn identify_across_exact_segment_boundaries_omits_outer_empty_segments() {
        let mut region = contiguous([
            Segment::unidentified(unidentified(b"aaaaaaaaaa", 0)),
            Segment::unidentified(unidentified(b"bbbbbbbbbb", 0)),
            Segment::unidentified(unidentified(b"cccccccccc", 0)),
        ]);

        region.identify(10, Node(10)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(unidentified(b"aaaaaaaaaa", 0)),
                Segment::identified(Node(10)),
                Segment::unidentified(unidentified(b"cccccccccc", 0)),
            ]),
        );
    }

    #[test]
    fn identify_rejects_range_that_overlaps_identified_segment() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));

        region.identify(3, Node(4)).unwrap();

        let original = region.clone();

        assert_eq!(
            region.identify(2, Node(3)),
            Err(Error::AlreadyIdentified(1)),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn identify_rejects_out_of_bounds_start() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));
        let original = region.clone();

        assert_eq!(
            region.identify(10, Node(1)),
            Err(Error::OutOfBounds(10, 10)),
        );
        assert_eq!(region, original);
    }

    #[test]
    fn identify_rejects_arange_that_extends_past_region() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));
        let original = region.clone();

        assert_eq!(region.identify(8, Node(3)), Err(Error::OutOfBounds(8, 10)));
        assert_eq!(region, original);
    }

    #[test]
    fn identify_allows_empty_region_at_addressable_offset() {
        let mut region = Contiguous::unidentified(unidentified(b"0123456789", 0));

        region.identify(3, Node(0)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(unidentified(b"012", 0)),
                Segment::identified(Node(0)),
                Segment::unidentified(unidentified(b"3456789", 0)),
            ]),
        );
    }

    #[test]
    fn identify_preserves_uninitialized_memory() {
        let mut region =
            Contiguous::unidentified(Unidentified::new(Bytes::from_static(b"abcd"), 6).unwrap());

        region.identify(2, Node(5)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(unidentified(b"ab", 0)),
                Segment::identified(Node(5)),
                Segment::unidentified(uninitialized(3)),
            ]),
        );
    }

    #[test]
    fn identify_can_start_in_uninitialized_memory() {
        let mut region = Contiguous::unidentified(unidentified(b"abcd", 6));

        region.identify(6, Node(2)).unwrap();

        assert_eq!(
            region,
            contiguous([
                Segment::unidentified(unidentified(b"abcd", 2)),
                Segment::identified(Node(2)),
                Segment::unidentified(uninitialized(2)),
            ]),
        );
    }

    #[test]
    fn segments_select_skips_empty_markers_at_range_boundaries() {
        let region = contiguous([
            Segment::unidentified(unidentified(b"aaaaa", 0)),
            Segment::identified(Node(0)),
            Segment::unidentified(unidentified(b"bbbbb", 0)),
            Segment::identified(Node(0)),
            Segment::unidentified(unidentified(b"ccccc", 0)),
        ]);

        let start_indices = region
            .segments()
            .select(5, 5)
            .map(|entry| entry.segment_index())
            .collect::<Vec<_>>();

        assert_eq!(start_indices, [2]);

        let crossing_indices = region
            .segments()
            .select(4, 2)
            .map(|entry| entry.segment_index())
            .collect::<Vec<_>>();

        assert_eq!(crossing_indices, [0, 1, 2]);
    }
}
