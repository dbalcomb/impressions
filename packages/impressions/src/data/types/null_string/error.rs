/// The null string error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered reading bytes.
    #[error("read error")]
    Read(#[from] bytes::TryGetError),

    /// A problem was encountered getting a UTF-8 string.
    #[error("utf-8 error")]
    Utf8(#[from] std::str::Utf8Error),

    /// A null byte was not found in the string.
    #[error("missing null byte")]
    MissingNull,
}
