/// The initialized region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the initialized region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// The requested slice is outside the initialized region.
    #[error(transparent)]
    SliceBounds(#[from] crate::memory::SliceBoundsError),
}
