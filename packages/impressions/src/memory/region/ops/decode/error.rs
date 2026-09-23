/// An error decoding a region.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Indicates that there are insufficient bytes available to decode the
    /// requested data.
    #[error("Insufficient bytes: requested {requested}, available {available}")]
    InsufficientBytes {
        /// The number of bytes requested.
        requested: usize,

        /// The number of bytes available.
        available: usize,
    },
}

/// An error decoding an array.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Array decode error at index {index} of {N}")]
pub struct ArrayDecodeError<T, const N: usize> {
    /// The index of the array where the error occurred.
    pub index: usize,

    /// The inner error at the index.
    #[source]
    pub error: T,
}

impl<const N: usize> ArrayDecodeError<Error, N> {
    /// Converts an array decode error into a decode error.
    ///
    /// This conversion is only correct when the inner error represents decoding
    /// the entire array item, such as a `u8`, and not an individual field of
    /// a larger structure.
    pub const fn into_decode_error(self) -> Error {
        match self.error {
            Error::InsufficientBytes {
                requested,
                available,
            } => Error::InsufficientBytes {
                requested: requested * N,
                available: requested * self.index + available % requested,
            },
        }
    }
}
