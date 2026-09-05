use crate::memory::address::Address;

/// The sparse region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The size of the sparse region exceeds the maximum region size.
    #[error("the region size {0} exceeds maximum {max}", max = u32::MAX as u64 + 1)]
    SizeTooLarge(u64),

    /// A segment has no addressable offset within the region.
    #[error("the segment at index {0} has no addressable offset")]
    UnaddressableSegment(usize),

    /// The address is out of bounds.
    #[error("the address {0} is out of bounds for size {1}")]
    OutOfBounds(Address, u64),

    /// The segment at the given index is already occupied.
    #[error("the segment at index {0} is already occupied")]
    AlreadyOccupied(usize),

    /// The uninitialized region is invalid.
    #[error("invalid uninitialized region")]
    Uninitialized(#[from] crate::memory::regions::uninitialized::Error),
}
