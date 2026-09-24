/// The hint/name entry error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Indicates that the hint is invalid.
    #[error("invalid hint")]
    Hint(#[from] crate::memory::region::ops::decode::Error),

    /// Indicates that the name is invalid.
    #[error("invalid name")]
    Name(
        #[from]
        crate::memory::region::types::aligned::Error<
            crate::memory::region::types::null_string::Error,
        >,
    ),
}
