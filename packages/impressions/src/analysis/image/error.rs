/// The image file error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An invalid signature was detected.
    #[error("Invalid signature")]
    InvalidSignature,

    /// An unsupported architecture was detected.
    #[error("Unsupported architecture")]
    UnsupportedArchitecture,

    /// A problem was encountered parsing the image.
    #[error("Parse error")]
    Parse(#[from] bytes::TryGetError),

    /// A problem was encountered reading the image.
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}
