/// An instruction error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with the prefix.
    #[error("prefix error")]
    Prefix(#[from] super::prefix::Error),

    /// A problem was encountered with the operation.
    #[error("operation error")]
    Operation(#[from] super::operation::Error),
}
