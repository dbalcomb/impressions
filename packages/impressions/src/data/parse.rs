use bytes::Buf;

/// Defines the ability to parse a data structure from a buffer.
pub trait Parse: Sized {
    /// The associated parse error.
    type Error;

    /// Parses the data from the given buffer.
    fn parse(buffer: impl Buf) -> Result<Self, Self::Error>;
}
