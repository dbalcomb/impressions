use std::ops::Deref;

use crate::memory::Extent;

/// A reference to a segment in a segmented region of memory.
#[derive(Clone, Debug)]
pub struct SegmentRef<'a, T> {
    segment: &'a T,
    index: usize,
    start_offset: u32,
    offset: u32,
}

impl<'a, T> SegmentRef<'a, T> {
    /// Constructs a new segment reference.
    pub(super) const fn new(segment: &'a T, index: usize, start_offset: u32) -> Self {
        Self {
            segment,
            index,
            start_offset,
            offset: start_offset,
        }
    }

    /// Builds the segment reference with an offset.
    pub(super) const fn with_offset(mut self, offset: u32) -> Self {
        self.set_offset(offset);
        self
    }

    /// Sets the offset for the segment reference.
    pub(super) const fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }
}

impl<'a, T> SegmentRef<'a, T> {
    /// Gets the referenced segment.
    pub const fn segment(&self) -> &'a T {
        self.segment
    }

    /// Gets the index for the referenced segment.
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Gets the offset for the referenced segment.
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Gets the start offset for the referenced segment.
    pub const fn start_offset(&self) -> u32 {
        self.start_offset
    }

    /// Gets the relative offset for the referenced segment.
    pub const fn relative_offset(&self) -> u32 {
        self.offset - self.start_offset
    }
}

impl<T> SegmentRef<'_, T>
where
    T: Extent,
{
    /// Checks whether the given offset is contained within the segment.
    pub fn contains_offset(&self, offset: u32) -> bool {
        let start = self.start_offset() as u64;
        let end = start + self.segment().size();

        (offset as u64) >= start && (offset as u64) < end
    }
}

impl<T> Deref for SegmentRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.segment()
    }
}
