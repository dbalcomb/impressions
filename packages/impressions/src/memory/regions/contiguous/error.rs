use crate::memory::address::AddressSpace;

/// The contiguous region error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the contiguous region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// An invalid address space was specified for the contiguous region.
    #[error("invalid address space")]
    AddressSpace(#[from] crate::memory::address::space::Error),

    /// The address space is outside the contiguous region.
    #[error("address space {0} is outside contiguous region {1}")]
    OutOfBounds(AddressSpace, AddressSpace),

    /// The segment at the given index is already identified.
    #[error("the segment at index {0} is already identified")]
    AlreadyIdentified(usize),

    /// The unidentified region is invalid.
    #[error("invalid unidentified region")]
    Unidentified(#[from] crate::memory::regions::unidentified::Error),
}
