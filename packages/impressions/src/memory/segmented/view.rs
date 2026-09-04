use crate::memory::Extent;

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
    /// Gets the segment at the given offset.
    pub fn get(&self, offset: u32) -> Option<SegmentRef<'a, T>> {
        self.iter().get(offset)
    }

    /// Gets a view over the complete segments that overlap with the given
    /// range.
    ///
    /// This method rebases the segments so that the first segment in the view
    /// has an index and offset of 0. The returned view may include segments
    /// that overlap with the requested range, but do not fully fit within it.
    pub fn range(&self, offset: u32, size: u64) -> Self {
        if size == 0 {
            return Self { segments: &[] };
        }

        let mut iter = self.iter();

        let Some(start) = iter.get(offset) else {
            return Self { segments: &[] };
        };

        if size <= u32::MAX as u64
            && let Some(end_offset) = offset.checked_add(size as u32)
            && let Some(end) = iter.get(end_offset)
        {
            if end_offset == end.start_offset() {
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

    fn get(&self, offset: u32) -> Option<SegmentRef<'_, Self::Segment>> {
        self.get(offset)
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
            .range(4, 2)
            .into_iter()
            .map(|segment| segment.index())
            .collect::<Vec<_>>();

        assert_eq!(indices, [0, 1]);
    }

    #[test]
    fn subview_excludes_segment_at_exclusive_end() {
        let nodes = [Node(5), Node(5), Node(5)];
        let indices = segments(&nodes)
            .range(0, 5)
            .into_iter()
            .map(|segment| segment.index())
            .collect::<Vec<_>>();

        assert_eq!(indices, [0]);
    }

    #[test]
    fn subview_rebases_offsets_and_indices() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes).range(5, 5);
        let segment = subview.get(0).unwrap();

        assert_eq!(segment.index(), 0);
        assert_eq!(segment.start_offset(), 0);
        assert_eq!(segment.offset(), 0);
        assert_eq!(subview.size(), 5);
    }

    #[test]
    fn nested_subviews_rebase_to_their_immediate_view() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes).range(4, 11).range(1, 5);
        let entries = subview
            .into_iter()
            .map(|segment| (segment.index(), segment.start_offset()))
            .collect::<Vec<_>>();

        assert_eq!(entries, [(0, 0), (1, 5)]);
    }

    #[test]
    fn subview_with_zero_size_is_empty() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes).range(5, 0);

        assert_eq!(subview.size(), 0);
        assert_eq!(subview.into_iter().count(), 0);
    }

    #[test]
    fn subview_outside_address_space_is_empty() {
        let nodes = [Node(5), Node(5), Node(5)];
        let subview = segments(&nodes).range(15, 1);

        assert_eq!(subview.size(), 0);
        assert_eq!(subview.into_iter().count(), 0);
    }
}
