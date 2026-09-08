//! Memory region slicing.

mod error;

use super::address::AddressSpace;

pub use self::error::Error;

/// Defines the ability to slice a region of memory.
pub trait Slice: Sized {
    /// The associated error type for slicing operations.
    type Error;

    /// Slices the region at the given address space.
    fn slice(&self, address_space: AddressSpace) -> Result<Self, Self::Error>;
}
