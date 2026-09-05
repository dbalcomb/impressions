use std::ops::Deref;

use crate::memory::Extent;
use crate::memory::address::Address;

/// A reference to a segment in a segmented region of memory.
#[derive(Debug)]
pub struct SegmentRef<'a, T> {
    segment: &'a T,
    index: usize,
    start_address: Address,
    address: Address,
}

impl<'a, T> SegmentRef<'a, T> {
    /// Constructs a new segment reference.
    pub(super) const fn new(segment: &'a T, index: usize, start_address: Address) -> Self {
        Self {
            segment,
            index,
            start_address,
            address: start_address,
        }
    }

    /// Builds the segment reference with the given address.
    pub(super) const fn with_address(mut self, address: Address) -> Self {
        self.set_address(address);
        self
    }

    /// Sets the address for the segment reference.
    pub(super) const fn set_address(&mut self, address: Address) {
        self.address = address;
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

    /// Gets the address for the referenced segment.
    pub const fn address(&self) -> Address {
        self.address
    }

    /// Gets the start address for the referenced segment.
    pub const fn start_address(&self) -> Address {
        self.start_address
    }

    /// Gets the relative address for the referenced segment.
    pub const fn relative_address(&self) -> Address {
        Address::new(self.address.offset(self.start_address))
    }
}

impl<T> SegmentRef<'_, T>
where
    T: Extent,
{
    /// Checks whether the given address is contained within the segment.
    pub fn contains_address(&self, address: Address) -> bool {
        let start = self.start_address().value() as u64;
        let end = start + self.segment().size();

        (address.value() as u64) >= start && (address.value() as u64) < end
    }
}

impl<T> Clone for SegmentRef<'_, T> {
    fn clone(&self) -> Self {
        Self {
            segment: self.segment,
            index: self.index,
            start_address: self.start_address,
            address: self.address,
        }
    }
}

impl<T> Deref for SegmentRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.segment()
    }
}
