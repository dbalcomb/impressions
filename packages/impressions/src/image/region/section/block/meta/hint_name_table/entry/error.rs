/// The hint/name entry error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Indicates that the hint is invalid.
    #[error("invalid hint")]
    Hint(#[from] bytes::TryGetError),

    /// Indicates that the name is invalid.
    #[error("invalid name")]
    Name(#[from] crate::data::types::null_string::Error),
}
