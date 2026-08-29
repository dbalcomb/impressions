/// The contiguous region error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The size of the contiguous region exceeds the maximum region size.
    #[error("the region size {0} exceeds maximum {max}", max = u32::MAX as u64 + 1)]
    SizeTooLarge(u64),

    /// A segment has no addressable offset within the region.
    #[error("the segment at index {0} has no addressable offset")]
    UnaddressableSegment(usize),

    /// The offset is out of bounds.
    #[error("the offset {0} is out of bounds for size {1}")]
    OutOfBounds(u32, u64),

    /// The segment at the given index is already identified.
    #[error("the segment at index {0} is already identified")]
    AlreadyIdentified(usize),

    /// The unidentified region is invalid.
    #[error("invalid unidentified region")]
    Unidentified(#[from] crate::memory::regions::unidentified::Error),
}
