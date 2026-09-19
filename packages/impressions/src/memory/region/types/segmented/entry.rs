use std::ops::Deref;

use crate::memory::address::{Address, AddressSpace};
use crate::memory::extent::Extent;

/// A reference to a segment in a segmented region of memory.
#[derive(Debug)]
pub struct SegmentRef<'a, T> {
    segment: &'a T,
    index: usize,
    address: Address,
    offset: u32,
}

impl<'a, T> SegmentRef<'a, T> {
    /// Constructs a new segment reference.
    pub(super) const fn new(segment: &'a T, index: usize, address: Address) -> Self {
        Self {
            segment,
            index,
            address,
            offset: 0,
        }
    }

    /// Builds the segment reference with the given offset address.
    pub(super) const fn with_offset_address(mut self, address: Address) -> Self {
        self.set_offset_address(address);
        self
    }

    /// Sets the offset address for the segment reference.
    pub(super) const fn set_offset_address(&mut self, address: Address) {
        self.offset = self.address.offset(address);
    }
}

impl<'a, T> SegmentRef<'a, T> {
    /// Gets the referenced segment.
    pub const fn segment(&self) -> &'a T {
        self.segment
    }

    /// Gets the index of the referenced segment.
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Gets the address of the referenced segment.
    pub const fn address(&self) -> Address {
        self.address
    }

    /// Gets the offset in the referenced segment.
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Gets the offset address in the referenced segment.
    ///
    /// This method returns the address of the offset within the segment, which
    /// may differ from the base address of the segment itself.
    pub const fn offset_address(&self) -> Address {
        Address::new(self.address.value() + self.offset)
    }
}

impl<T> SegmentRef<'_, T>
where
    T: Extent,
{
    /// Gets the address space of the referenced segment.
    pub fn address_space(&self) -> AddressSpace {
        self.address()
            .to_space(self.segment.size())
            .expect("valid address space")
    }
}

impl<T> Clone for SegmentRef<'_, T> {
    fn clone(&self) -> Self {
        Self {
            segment: self.segment,
            index: self.index,
            address: self.address,
            offset: self.offset,
        }
    }
}

impl<T> Deref for SegmentRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.segment()
    }
}
