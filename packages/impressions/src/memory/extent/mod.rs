//! Memory size representation and manipulation.

mod error;
mod size;

pub use self::error::Error;
pub use self::size::Size;

use super::address::AddressSpace;

/// Defines the size of a memory region.
pub trait Extent {
    /// Gets the size of the region.
    fn size(&self) -> Size;

    /// Gets the address space of the region.
    fn address_space(&self) -> AddressSpace {
        self.size().to_address_space()
    }
}
