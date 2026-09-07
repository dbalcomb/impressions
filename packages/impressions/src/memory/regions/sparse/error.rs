use crate::memory::Size;
use crate::memory::address::Address;

/// The sparse region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with a size value.
    #[error("size error")]
    Size(#[from] crate::memory::SizeError),

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
