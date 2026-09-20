/// An aligned region error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error<T> {
    /// The inner region could not be parsed.
    #[error("region error")]
    Region(#[source] T),

    /// The trailing padding could not be read.
    #[error("padding error")]
    Padding(#[from] bytes::TryGetError),

    /// The trailing padding was not null.
    #[error("expected null padding byte but got {0:02x}")]
    NonNullPadding(u8),

    /// The requested alignment is invalid.
    #[error("alignment must be non-zero")]
    ZeroAlignment,
}
