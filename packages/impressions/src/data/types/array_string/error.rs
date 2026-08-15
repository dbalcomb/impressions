/// The array string error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A problem was encountered reading bytes.
    #[error("Read error")]
    Read(#[from] bytes::TryGetError),

    /// A problem was encountered getting a UTF-8 string.
    #[error("UTF-8 error")]
    Utf8(#[from] std::str::Utf8Error),
}
