/// A prefix error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered decoding a prefix.
    #[error("decode error")]
    Decode(#[from] crate::memory::region::ops::decode::Error),

    /// An invalid prefix was encountered.
    #[error("invalid prefix {0:02x}")]
    Invalid(u8),

    /// Too many prefixes were encountered.
    #[error("too many prefixes")]
    TooManyPrefixes,
}
