use crate::memory::extent::Size;

use super::Position;

/// A memory region cursor error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Indicates that the position is out of bounds.
    #[error("the position {0} is out of bounds for size {1}")]
    OutOfBounds(Position, Size),

    /// Indicates that the position cannot be advanced by the offset.
    #[error("unable to advance {1} from position {0}")]
    CannotAdvance(Position, u32),
}

/// A memory region cursor read error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ReadError<P, C = Error> {
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
