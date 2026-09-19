/// An imported DLL name error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The DLL name could not be read.
    #[error("invalid name")]
    Name(#[from] crate::data::types::null_string::Error),

    /// The alignment padding could not be read.
    #[error("invalid padding")]
    Padding(#[from] bytes::TryGetError),
}
