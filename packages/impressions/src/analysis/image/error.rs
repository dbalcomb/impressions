use crate::data::types::array_string;
use crate::memory::address::AddressSpaceError;

/// The image file error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An invalid signature was detected.
    #[error("Invalid signature")]
    InvalidSignature,

    /// An unsupported architecture was detected.
    #[error("Unsupported architecture")]
    UnsupportedArchitecture,

    /// An empty section was detected.
    #[error("Empty section")]
    EmptySection,

    /// A problem was encountered parsing the image.
    #[error("Parse error")]
    Parse(#[from] bytes::TryGetError),

    /// A problem was encountered reading the image.
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// A problem was encountered reading a UTF-8 string.
    #[error("UTF-8 error")]
    Utf8(#[from] std::str::Utf8Error),

    /// A problem was encountered with an address space.
    #[error("Address space error")]
    AddressSpace(#[from] AddressSpaceError),
}

impl From<array_string::Error> for Error {
    fn from(err: array_string::Error) -> Self {
        match err {
            array_string::Error::Read(err) => Self::Parse(err),
            array_string::Error::Utf8(err) => Self::Utf8(err),
        }
    }
}
