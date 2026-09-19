/// The sparse region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the sparse region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// An invalid address space was specified for the contiguous region.
    #[error("invalid address space")]
    AddressSpace(#[from] crate::memory::address::space::Error),

    /// The uninitialized region is invalid.
    #[error("invalid uninitialized region")]
    Uninitialized(#[from] crate::memory::region::types::uninitialized::Error),

    /// A problem was encountered inserting a region.
    #[error("insert operation error")]
    Insert(#[from] crate::memory::region::ops::insert::Error),
}
