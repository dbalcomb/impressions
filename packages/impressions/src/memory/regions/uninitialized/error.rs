/// The uninitialized region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The requested slice is invalid.
    #[error("invalid slice")]
    Slice(#[from] crate::memory::slice::Error),
}
