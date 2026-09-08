/// The initialized region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid size was specified for the initialized region.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// The requested slice is invalid.
    #[error("invalid slice")]
    Slice(#[from] crate::memory::slice::Error),
}
