/// The image file headers error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid signature was detected.
    #[error("invalid signature")]
    InvalidSignature,

    /// An unsupported architecture was detected.
    #[error("unsupported architecture")]
    UnsupportedArchitecture,

    /// An unsupported data directory count was detected.
    #[error("unsupported data directory count: {0}")]
    UnsupportedDataDirectoryCount(u32),

    /// An invalid section name was detected.
    #[error("invalid section name")]
    InvalidSectionName(#[source] crate::memory::region::types::array_string::Error),

    /// A problem was encountered decoding the image.
    #[error("decode error")]
    Decode(#[from] crate::memory::region::ops::decode::Error),

    /// A problem was encountered with an unidentified region.
    #[error("unidentified region error")]
    Unidentified(#[from] crate::memory::region::types::unidentified::Error),

    /// A problem was encountered with a contiguous region.
    #[error("contiguous region error")]
    Contiguous(#[from] crate::memory::region::types::contiguous::Error),
}
