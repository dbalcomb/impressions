/// The unidentified region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The uninitialized region is not the last segment.
    #[error("the uninitialized region is not the last segment")]
    UninitializedNotLast,

    /// The uninitialized region is already present.
    #[error("the uninitialized region is already present")]
    UninitializedAlreadyPresent,

    /// A problem was encountered with a size value.
    #[error("size error")]
    Size(#[from] crate::memory::SizeError),

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
