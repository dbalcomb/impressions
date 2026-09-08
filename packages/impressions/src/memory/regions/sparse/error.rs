use crate::memory::address::AddressSpace;

/// The sparse region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the sparse region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// An invalid address space was specified for the contiguous region.
    #[error("invalid address space")]
    AddressSpace(#[from] crate::memory::address::space::Error),

    /// The address space is outside the sparse region.
    #[error("address space {0} is outside sparse region {1}")]
    OutOfBounds(AddressSpace, AddressSpace),

    /// The segment at the given index is already occupied.
    #[error("the segment at index {0} is already occupied")]
    AlreadyOccupied(usize),

    /// The uninitialized region is invalid.
    #[error("invalid uninitialized region")]
    Uninitialized(#[from] crate::memory::regions::uninitialized::Error),
}
