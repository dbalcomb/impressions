/// A memory region encoding error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Indicates that the encoded regions do not form a valid file layout.
    #[error("The encoded regions do not form a valid file layout.")]
    InvalidLayout,

    /// An I/O error occurred while writing encoded bytes.
    #[error("An I/O error occurred while writing encoded bytes.")]
    Io(#[from] std::io::Error),
}
