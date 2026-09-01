//! A memory region that is segmented into multiple sub-regions.

mod entry;
mod iter;

use super::Extent;

pub use self::entry::SegmentRef;
pub use self::iter::Segments;

/// Defines a memory region that is segmented into multiple sub-regions.
pub trait Segmented: Extent + Sized {
    /// The associated segment type for this region.
    type Segment: Extent;

    /// Gets an iterator over the segments.
    fn segments(&self) -> Segments<'_, Self>;

    /// Gets the segment at the given offset.
    fn get(&self, offset: u32) -> Option<SegmentRef<'_, Self>> {
        self.segments().get(offset)
    }
}
