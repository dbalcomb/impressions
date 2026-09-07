use crate::memory::address::Address;
use crate::memory::extent::Size;

/// The sparse region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the sparse region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// The address is out of bounds.
    #[error("the address {0} is out of bounds for size {1}")]
    OutOfBounds(Address, Size),

    /// The segment at the given index is already occupied.
    #[error("the segment at index {0} is already occupied")]
    AlreadyOccupied(usize),

    /// The uninitialized region is invalid.
    #[error("invalid uninitialized region")]
    Uninitialized(#[from] crate::memory::regions::uninitialized::Error),
}
