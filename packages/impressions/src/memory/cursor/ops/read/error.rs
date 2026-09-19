/// A memory region cursor read error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error<P, C = crate::memory::cursor::Error> {
    /// Indicates that the region does not support reading.
    #[error("unsupported region")]
    Unsupported,

    /// Indicates a problem parsing the region.
    #[error("parse error")]
    Parse(#[source] P),

    /// Indicates a problem seeking the cursor.
    #[error("cursor error")]
    Cursor(#[source] C),
}
