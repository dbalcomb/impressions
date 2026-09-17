/// The contiguous region error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the contiguous region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// An invalid address space was specified for the contiguous region.
    #[error("invalid address space")]
    AddressSpace(#[from] crate::memory::address::space::Error),

    /// The unidentified region is invalid.
    #[error("invalid unidentified region")]
    Unidentified(#[from] crate::memory::regions::unidentified::Error),

    /// A problem was encountered inserting a region.
    #[error("insert operation error")]
    Insert(#[from] crate::memory::ops::insert::Error),
}
