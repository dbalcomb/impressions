/// The uninitialized region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The requested slice is outside the uninitialized region.
    #[error(transparent)]
    SliceBounds(#[from] crate::memory::SliceBoundsError),
}
