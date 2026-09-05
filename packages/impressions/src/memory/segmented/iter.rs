use std::iter::{Enumerate, FusedIterator};
use std::slice::Iter as SliceIter;

use crate::memory::Extent;
use crate::memory::address::Address;

use super::SegmentRef;

/// An iterator over the segments of a segmented region of memory.
#[derive(Debug)]
pub struct SegmentsIter<'a, T> {
    segments: Enumerate<SliceIter<'a, T>>,
    next_offset: u64,
    next_back_offset: u64,
}

impl<'a, T> SegmentsIter<'a, T>
where
    T: Extent,
{
    /// Creates an iterator over the segments of a segmented region of memory.
    pub(in crate::memory) fn new(segments: &'a [T]) -> Self {
        Self {
            segments: segments.iter().enumerate(),
            next_offset: 0,
            next_back_offset: segments.iter().map(Extent::size).sum(),
        }
    }
}

impl<'a, T> SegmentsIter<'a, T>
where
    T: Extent,
{
    /// Gets the segment at the given address.
    ///
    /// This method searches for the segment that contains the given address and
    /// stops iterating once it has been found.
    pub fn get(&mut self, address: Address) -> Option<SegmentRef<'a, T>> {
        self.find(|segment| segment.contains_address(address))
            .map(|segment| segment.with_address(address))
    }

    /// Gets the segment at the given address, starting from the back.
    ///
    /// This method searches for the segment that contains the given address and
    /// stops iterating once it has been found.
    pub fn get_back(&mut self, address: Address) -> Option<SegmentRef<'a, T>> {
        self.rfind(|segment| segment.contains_address(address))
            .map(|segment| segment.with_address(address))
    }
}

impl<'a, T> SegmentsIter<'a, T>
where
    T: Extent,
{
    /// Selects the segments that overlap with the given address and size.
    pub fn select(self, address: Address, size: u64) -> impl Iterator<Item = SegmentRef<'a, T>> {
        let end = u64::from(address.value()).saturating_add(size);

        self.skip_while(move |segment| !segment.contains_address(address))
            .take_while(move |segment| u64::from(segment.start_address().value()) < end)
    }
}

impl<T> Clone for SegmentsIter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            segments: self.segments.clone(),
            next_offset: self.next_offset,
            next_back_offset: self.next_back_offset,
        }
    }
}

impl<'a, T> Iterator for SegmentsIter<'a, T>
where
    T: Extent,
{
    type Item = SegmentRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, segment) = self.segments.next()?;
        let offset = self.next_offset;

        self.next_offset += segment.size();

        Some(SegmentRef::new(segment, index, Address::new(offset as u32)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.segments.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for SegmentsIter<'a, T>
where
    T: Extent,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let (index, segment) = self.segments.next_back()?;

        self.next_back_offset -= segment.size();

        Some(SegmentRef::new(
            segment,
            index,
            Address::new(self.next_back_offset as u32),
        ))
    }
}

impl<'a, T> ExactSizeIterator for SegmentsIter<'a, T>
where
    T: Extent,
{
    fn len(&self) -> usize {
        self.segments.len()
    }
}

impl<'a, T> FusedIterator for SegmentsIter<'a, T> where T: Extent {}
