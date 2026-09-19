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
