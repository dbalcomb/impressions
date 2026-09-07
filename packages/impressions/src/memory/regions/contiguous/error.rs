use crate::memory::address::Address;
use crate::memory::extent::Size;

/// The contiguous region error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the contiguous region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// The address is out of bounds.
    #[error("the address {0} is out of bounds for size {1}")]
    OutOfBounds(Address, Size),

    /// The segment at the given index is already identified.
    #[error("the segment at index {0} is already identified")]
    AlreadyIdentified(usize),

    /// The unidentified region is invalid.
    #[error("invalid unidentified region")]
    Unidentified(#[from] crate::memory::regions::unidentified::Error),
}
