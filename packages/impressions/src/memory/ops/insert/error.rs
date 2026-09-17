use crate::memory::address::{Address, AddressSpace};

/// An insert error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Indicates that the region does not support insertion at the address.
    #[error("unsupported region at {0}")]
    Unsupported(Address),

    /// Indicates that the inserted region is outside of the container region.
    #[error("the address space {0} is out of bounds for region {1}")]
    OutOfBounds(AddressSpace, AddressSpace),
}
