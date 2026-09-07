/// The initialized region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with a size value.
    #[error("size error")]
    Size(#[from] crate::memory::SizeError),

    /// The requested slice is outside the initialized region.
    #[error(transparent)]
    SliceBounds(#[from] crate::memory::SliceBoundsError),
}
