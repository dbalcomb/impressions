use std::ops::Deref;

use crate::memory::Extent;

use super::Segment;

/// An entry in a contiguous region of memory.
#[derive(Clone, Debug)]
pub struct Entry<'a, T> {
    segment: &'a Segment<T>,
    segment_index: usize,
    segment_offset: u32,
    offset: u32,
}

impl<'a, T> Entry<'a, T> {
    /// Constructs a new entry.
    pub(super) const fn new(
        segment: &'a Segment<T>,
        segment_index: usize,
        segment_offset: u32,
    ) -> Self {
        Self {
            segment,
            segment_index,
            segment_offset,
            offset: segment_offset,
        }
    }

    /// Builds the entry with an offset.
    pub(super) fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }
}

impl<'a, T> Entry<'a, T> {
    /// Gets the segment for this entry.
    pub const fn segment(&self) -> &'a Segment<T> {
        self.segment
    }

    /// Gets the segment index for this entry.
    pub const fn segment_index(&self) -> usize {
        self.segment_index
    }

    /// Gets the segment offset for this entry.
    pub const fn segment_offset(&self) -> u32 {
        self.segment_offset
    }

    /// Gets the offset for this entry.
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Gets the relative offset for this entry.
    pub const fn relative_offset(&self) -> u32 {
        self.offset - self.segment_offset
    }
}

impl<T> Entry<'_, T>
where
    T: Extent,
{
    /// Checks whether the given offset is contained within this entry.
    pub fn contains_offset(&self, offset: u32) -> bool {
        let start = self.segment_offset() as u64;
        let end = start + self.segment().size();

        (offset as u64) >= start && (offset as u64) < end
    }
}

impl<T> Deref for Entry<'_, T> {
    type Target = Segment<T>;

    fn deref(&self) -> &Self::Target {
        self.segment()
    }
}
