use crate::memory::address::Address;
use crate::memory::extent::Size;

/// A null-terminated table error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error<T> {
    /// Indicates that a table row could not be parsed.
    #[error("unable to parse table row {address}")]
    Row {
        /// The address of the row that failed to parse.
        address: Address,

        /// The error that occurred while parsing the row.
        #[source]
        error: T,
    },

    /// Indicates that the size of the table does not match the expected size.
    #[error("expected size {expected} but got {actual}")]
    SizeMismatch {
        /// The expected size of the table.
        expected: Size,

        /// The actual size of the table.
        actual: Size,
    },

    /// Indicates that the specified table size cannot contain whole rows.
    #[error("table size {size} is not a multiple of row size {row_size}")]
    SizeNotMultiple {
        /// The specified size of the table.
        size: Size,

        /// The fixed size of each row.
        row_size: Size,
    },

    /// Indicates that the table has no null terminator before its permitted
    /// size is exhausted.
    #[error("missing null terminator before table exceeds its permitted size")]
    MissingNullTerminator,
}
