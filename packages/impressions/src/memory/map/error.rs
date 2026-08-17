use crate::memory::address::AddressSpace;

/// A memory map error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The address space intersects another address space.
    #[error("Intersecting address spaces {0} and {1}")]
    Intersect(AddressSpace, AddressSpace),

    /// The address space is out of bounds.
    #[error("Address space {0} out of bounds {1}")]
    OutOfBounds(AddressSpace, AddressSpace),
}
