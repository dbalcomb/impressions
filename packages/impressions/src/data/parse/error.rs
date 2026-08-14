use bytes::TryGetError;
use thiserror::Error;

/// An error parsing an array.
#[derive(Debug, PartialEq, Eq, Error)]
#[error("Array parse error at index {index} of {N}")]
pub struct ArrayParseError<T, const N: usize> {
    /// The index of the array where the error occurred.
    pub index: usize,

    /// The inner error at the index.
    #[source]
    pub error: T,
}

impl<const N: usize> ArrayParseError<TryGetError, N> {
    /// Converts an array parse error into a buffer error.
    ///
    /// This conversion is only correct when the inner error represents parsing
    /// the entire array item, such as a `u8`, and not an individual field of
    /// a larger structure.
    pub const fn into_buffer_error(self) -> TryGetError {
        TryGetError {
            requested: self.error.requested * N,
            available: self.error.requested * self.index
                + self.error.available % self.error.requested,
        }
    }
}
