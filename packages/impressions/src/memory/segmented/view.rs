use crate::memory::address::{Address, AddressSpace};
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};

use super::{SegmentRef, Segmented, SegmentsCursor, SegmentsIter};

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
        debug_assert!(!segments.is_empty());

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
    /// address space.
    ///
    /// This method rebases the segments so that the first segment in the view
    /// has an index and address of 0. The returned view may include segments
    /// that overlap with the requested range, but do not fully fit within it.
    pub fn range(&self, address_space: AddressSpace) -> Option<Self> {
        let mut iter = self.iter();

        let start = iter.get(address_space.first())?;

        if let Some(end_address) = address_space.next()
            && let Some(end) = iter.get(end_address)
        {
            if end_address == end.address() {
                return Some(Self {
                    segments: &self.segments[start.index()..end.index()],
                });
            }

            return Some(Self {
                segments: &self.segments[start.index()..=end.index()],
            });
        }

        Some(Self {
            segments: &self.segments[start.index()..],
        })
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
    fn size(&self) -> Size {
        Size::try_sum(self.segments.iter().map(Extent::size))
            .expect("sum of sizes does not exceed maximum size")
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

impl<'a, T> AsCursor for Segments<'a, T>
where
    T: Extent + AsCursor,
{
    #[rustfmt::skip]
    type Cursor<'b> = SegmentsCursor<'a, T>
    where
        Self: 'b;

    fn cursor(&self) -> Self::Cursor<'_> {
        SegmentsCursor::new(self.clone())
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
    use crate::memory::address::Address;
    use crate::memory::extent::{Extent, Size};

    use super::Segments;

    struct Node(u64);

    impl Extent for Node {
        fn size(&self) -> Size {
            Size::new(self.0).unwrap()
        }
    }

    fn segments(nodes: &[Node]) -> Segments<'_, Node> {
        Segments::new(nodes)
    }

    #[test]
    fn subview_includes_segments_overlapping_requested_range() {
        let nodes = [Node(5), Node(5), Node(5)];
        let indices = segments(&nodes)
            .range(Address::new(4).to_space(Size::new(2).unwrap()).unwrap())
            .unwrap()
            .into_iter()
            .map(|segment| segment.index())
            .collect::<Vec<_>>();

        assert_eq!(indices, [0, 1]);
    }

    #[test]
    fn subview_excludes_segment_at_exclusive_end() {
        let nodes = [Node(5), Node(5), Node(5)];
        let indices = segments(&nodes)
            .range(Address::new(0).to_space(Size::new(5).unwrap()).unwrap())
            .unwrap()
            .into_iter()
            .map(|segment| segment.index())
            .collect::<Vec<_>>();

        assert_eq!(indices, [0]);
    }

    #[test]
    fn subview_rebases_address_and_index() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes)
            .range(Address::new(5).to_space(Size::new(5).unwrap()).unwrap())
            .unwrap();
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
            .range(Address::new(4).to_space(Size::new(11).unwrap()).unwrap())
            .unwrap()
            .range(Address::new(1).to_space(Size::new(5).unwrap()).unwrap())
            .unwrap();
        let entries = subview
            .into_iter()
            .map(|segment| (segment.index(), segment.address()))
            .collect::<Vec<_>>();

        assert_eq!(entries, [(0, Address::new(0)), (1, Address::new(5))]);
    }

    #[test]
    fn subview_outside_address_space_is_none() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview =
            segments(&nodes).range(Address::new(15).to_space(Size::new(1).unwrap()).unwrap());

        assert!(subview.is_none());
    }
}
