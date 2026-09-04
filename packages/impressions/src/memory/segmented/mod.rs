//! A memory region that is segmented into multiple sub-regions.

mod entry;
mod iter;
mod view;

use super::Extent;

pub use self::entry::SegmentRef;
pub use self::iter::SegmentsIter;
pub use self::view::Segments;

/// Defines a memory region that is segmented into multiple sub-regions.
pub trait Segmented: Extent {
    /// The associated segment type for this region.
    type Segment: Extent;

    /// Gets an iterator over the segments.
    fn segments(&self) -> Segments<'_, Self::Segment>;

    /// Gets the segment at the given offset.
    fn get(&self, offset: u32) -> Option<SegmentRef<'_, Self::Segment>> {
        self.segments().get(offset)
    }
}
