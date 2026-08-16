//! Memory region representation and manipulation.

use super::address::{Address, AddressSpace};

/// Defines a memory region.
///
/// A memory region is a contiguous block of memory that can be used for storing
/// data or code. The region is defined by its address space, which specifies
/// the range of addresses that the region occupies.
pub trait Region {
    /// Gets the address space of the region.
    fn address_space(&self) -> AddressSpace;

    /// Gets the address of the region.
    ///
    /// This method is simply a shorthand to get the first address from the
    /// address space and should return the same value.
    fn address(&self) -> Address {
        self.address_space().first()
    }

    /// Gets the size of the region.
    ///
    /// This method is simply a shorthand to get the size from the address
    /// space and should return the same value.
    fn size(&self) -> u64 {
        self.address_space().size()
    }
}
