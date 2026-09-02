/// The image file headers error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An invalid signature was detected.
    #[error("invalid signature")]
    InvalidSignature,

    /// An unsupported architecture was detected.
    #[error("unsupported architecture")]
    UnsupportedArchitecture,

    /// An invalid section name was detected.
    #[error("invalid section name")]
    InvalidSectionName(crate::data::types::array_string::Error),

    /// A problem was encountered parsing the image.
    #[error("parse error")]
    Parse(#[from] bytes::TryGetError),

    /// A problem was encountered with an unidentified region.
    #[error("unidentified region error")]
    Unidentified(#[from] crate::memory::regions::unidentified::Error),

    /// A problem was encountered with a contiguous region.
    #[error("contiguous region error")]
    Contiguous(#[from] crate::memory::regions::contiguous::Error),
}
