use crate::memory::Extent;
use crate::memory::address::Address;

use super::{SegmentRef, Segmented, SegmentsIter};

/// A view over the segments of a segmented region of memory.
#[derive(Debug, PartialEq, Eq)]
pub struct Segments<'a, T> {
    segments: &'a [T],
}

impl<'a, T> Segments<'a, T>
where
    T: Extent,
{
    /// Constructs a new view over the given segments.
    pub(in crate::memory) fn new(segments: &'a [T]) -> Self {
        Self { segments }
    }
}

impl<'a, T> Segments<'a, T>
where
    T: Extent,
{
    /// Gets the segment at the given address.
    pub fn get(&self, address: Address) -> Option<SegmentRef<'a, T>> {
        self.iter().get(address)
    }

    /// Gets a view over the complete segments that overlap with the given
    /// range.
    ///
    /// This method rebases the segments so that the first segment in the view
    /// has an index and address of 0. The returned view may include segments
    /// that overlap with the requested range, but do not fully fit within it.
    pub fn range(&self, address: Address, size: u64) -> Self {
        if size == 0 {
            return Self { segments: &[] };
        }

        let mut iter = self.iter();

        let Some(start) = iter.get(address) else {
            return Self { segments: &[] };
        };

        if size <= u32::MAX as u64
            && let Some(end_address) = address.checked_add(size as u32)
            && let Some(end) = iter.get(end_address)
        {
            if end_address == end.address() {
                return Self {
                    segments: &self.segments[start.index()..end.index()],
                };
            }

            return Self {
                segments: &self.segments[start.index()..=end.index()],
            };
        }

        Self {
            segments: &self.segments[start.index()..],
        }
    }

    /// Gets an iterator over the segments.
    pub fn iter(&self) -> SegmentsIter<'a, T> {
        SegmentsIter::new(self.segments)
    }
}

impl<'a, T> Extent for Segments<'a, T>
where
    T: Extent,
{
    fn size(&self) -> u64 {
        self.segments.iter().map(Extent::size).sum()
    }
}

impl<T> Segmented for Segments<'_, T>
where
    T: Extent,
{
    type Segment = T;

    fn segments(&self) -> Segments<'_, T> {
        self.clone()
    }

    fn get(&self, address: Address) -> Option<SegmentRef<'_, Self::Segment>> {
        self.get(address)
    }
}

impl<'a, T> Clone for Segments<'a, T> {
    fn clone(&self) -> Self {
        Self {
            segments: self.segments,
        }
    }
}

impl<'a, T> IntoIterator for Segments<'a, T>
where
    T: Extent,
{
    type Item = SegmentRef<'a, T>;
    type IntoIter = SegmentsIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SegmentsIter::new(self.segments)
    }
}

impl<'a, T> IntoIterator for &Segments<'a, T>
where
    T: Extent,
{
    type Item = SegmentRef<'a, T>;
    type IntoIter = SegmentsIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SegmentsIter::new(self.segments)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::Extent;
    use crate::memory::address::Address;

    use super::Segments;

    struct Node(u64);

    impl Extent for Node {
        fn size(&self) -> u64 {
            self.0
        }
    }

    fn segments(nodes: &[Node]) -> Segments<'_, Node> {
        Segments::new(nodes)
    }

    #[test]
    fn subview_includes_segments_overlapping_requested_range() {
        let nodes = [Node(5), Node(5), Node(5)];
        let indices = segments(&nodes)
            .range(Address::new(4), 2)
            .into_iter()
            .map(|segment| segment.index())
            .collect::<Vec<_>>();

        assert_eq!(indices, [0, 1]);
    }

    #[test]
    fn subview_excludes_segment_at_exclusive_end() {
        let nodes = [Node(5), Node(5), Node(5)];
        let indices = segments(&nodes)
            .range(Address::new(0), 5)
            .into_iter()
            .map(|segment| segment.index())
            .collect::<Vec<_>>();

        assert_eq!(indices, [0]);
    }

    #[test]
    fn subview_rebases_address_and_index() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes).range(Address::new(5), 5);
        let segment = subview.get(Address::new(0)).unwrap();

        assert_eq!(segment.index(), 0);
        assert_eq!(segment.address(), Address::new(0));
        assert_eq!(segment.offset_address(), Address::new(0));
        assert_eq!(subview.size(), 5);
    }

    #[test]
    fn nested_subviews_rebase_to_their_immediate_view() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes)
            .range(Address::new(4), 11)
            .range(Address::new(1), 5);
        let entries = subview
            .into_iter()
            .map(|segment| (segment.index(), segment.address()))
            .collect::<Vec<_>>();

        assert_eq!(entries, [(0, Address::new(0)), (1, Address::new(5))]);
    }

    #[test]
    fn subview_with_zero_size_is_empty() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes).range(Address::new(5), 0);

        assert_eq!(subview.size(), 0);
        assert_eq!(subview.into_iter().count(), 0);
    }

    #[test]
    fn subview_outside_address_space_is_empty() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes).range(Address::new(15), 1);

        assert_eq!(subview.size(), 0);
        assert_eq!(subview.into_iter().count(), 0);
    }
}
