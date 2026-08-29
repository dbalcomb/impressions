/// The unidentified region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The region cannot be empty.
    #[error("the region cannot be empty")]
    Empty,

    /// The size of the unidentified region exceeds the maximum region size.
    #[error("the region size {0} exceeds maximum {max}", max = u32::MAX as u64 + 1)]
    SizeTooLarge(u64),

    /// The requested slice is outside the unidentified region.
    #[error(transparent)]
    SliceBounds(#[from] crate::memory::SliceBoundsError),

    /// The initialized region is invalid.
    #[error("the initialized region is invalid")]
    Initialized(#[from] crate::memory::regions::initialized::Error),

    /// The uninitialized region is invalid.
    #[error("the uninitialized region is invalid")]
    Uninitialized(#[from] crate::memory::regions::uninitialized::Error),
}
