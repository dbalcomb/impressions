use std::iter::{Enumerate, FusedIterator};
use std::slice::Iter as SliceIter;

use crate::memory::Extent;

use super::{Contiguous, Entry, Segment};

/// An iterator over the segments of a contiguous region of memory.
#[derive(Clone, Debug)]
pub struct Segments<'a, T> {
    segments: Enumerate<SliceIter<'a, Segment<T>>>,
    next_offset: u64,
    next_back_offset: u64,
}

impl<'a, T> Segments<'a, T>
where
    T: Extent,
{
    /// Creates an iterator over the segments of a contiguous region of memory.
    pub(super) fn new(segments: &'a Contiguous<T>) -> Self {
        Self {
            segments: segments.0.iter().enumerate(),
            next_offset: 0,
            next_back_offset: segments.size(),
        }
    }
}

impl<'a, T> Segments<'a, T>
where
    T: Extent,
{
    /// Gets the entry for the segment at the given offset.
    ///
    /// This method searches for the segment that contains the given offset and
    /// stops iterating once it has been found.
    pub fn get(&mut self, offset: u32) -> Option<Entry<'a, T>> {
        self.find(|entry| entry.contains_offset(offset))
            .map(|entry| entry.with_offset(offset))
    }

    /// Gets the entry for the segment at the given offset, searching from the
    /// back of the iterator.
    ///
    /// This method searches for the segment that contains the given offset and
    /// stops iterating once it has been found.
    pub fn get_back(&mut self, offset: u32) -> Option<Entry<'a, T>> {
        self.rfind(|entry| entry.contains_offset(offset))
            .map(|entry| entry.with_offset(offset))
    }
}

impl<'a, T> Segments<'a, T>
where
    T: Extent,
{
    /// Selects the entries that overlap with the given offset and size.
    pub fn select(self, offset: u32, size: u64) -> impl Iterator<Item = Entry<'a, T>> {
        let end = u64::from(offset).saturating_add(size);

        self.skip_while(move |entry| !entry.contains_offset(offset))
            .take_while(move |entry| u64::from(entry.segment_offset()) < end)
    }
}

impl<'a, T> Iterator for Segments<'a, T>
where
    T: Extent,
{
    type Item = Entry<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, segment) = self.segments.next()?;
        let offset = self.next_offset;

        self.next_offset += segment.size();

        Some(Entry::new(segment, index, offset as u32))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.segments.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Segments<'a, T>
where
    T: Extent,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let (index, segment) = self.segments.next_back()?;

        self.next_back_offset -= segment.size();

        Some(Entry::new(segment, index, self.next_back_offset as u32))
    }
}

impl<'a, T> ExactSizeIterator for Segments<'a, T>
where
    T: Extent,
{
    fn len(&self) -> usize {
        self.segments.len()
    }
}

impl<'a, T> FusedIterator for Segments<'a, T> where T: Extent {}
