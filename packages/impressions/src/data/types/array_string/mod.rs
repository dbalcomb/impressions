//! The array string data type.

mod error;

use std::fmt::{self, Debug, Display};
use std::ops::Deref;

use bytes::Buf;

use crate::data::parse::{ArrayParseError, Parse};

pub use self::error::Error;

/// A UTF-8 string backed by a fixed-size array.
///
/// This data type is internally represented by a fixed-sized byte array with
/// trailing null bytes trimmed on access. The length of the string can vary
/// up to the capacity of the array.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ArrayString<const N: usize>([u8; N]);

impl<const N: usize> ArrayString<N> {
    /// Gets the string slice.
    pub const fn as_str(&self) -> &str {
        match str::from_utf8(self.as_bytes()) {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    /// Gets the byte slice.
    pub const fn as_bytes(&self) -> &[u8] {
        trim_trailing_null(&self.0)
    }

    /// Gets the capacity of the string.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Gets the length of the string.
    pub const fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Checks whether the string is empty.
    pub const fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }
}

impl<const N: usize> Parse for ArrayString<N> {
    type Error = Error;

    fn parse(buffer: impl Buf) -> Result<Self, Error> {
        let bytes = <[u8; N]>::parse(buffer).map_err(ArrayParseError::into_buffer_error)?;

        str::from_utf8(trim_trailing_null(&bytes))?;

        Ok(Self(bytes))
    }
}

impl<const N: usize> PartialEq<str> for ArrayString<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> PartialEq<&str> for ArrayString<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> Display for ArrayString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl<const N: usize> Debug for ArrayString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> Deref for ArrayString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for ArrayString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<[u8]> for ArrayString<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Trims the trailing null bytes.
const fn trim_trailing_null(mut bytes: &[u8]) -> &[u8] {
    while let [rest @ .., 0] = bytes {
        bytes = rest;
    }

    bytes
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crate::data::parse::{Parse, TryGetError};

    use super::{ArrayString, Error};

    #[test]
    fn test_parse_exact() {
        let mut buffer = b"Hello".as_slice();
        let string = ArrayString::<5>::parse(&mut buffer).unwrap();

        assert_eq!(string, "Hello");
        assert_eq!(buffer, []);
    }

    #[test]
    fn test_parse_under() {
        let mut buffer = b"Hello World".as_slice();
        let string = ArrayString::<5>::parse(&mut buffer).unwrap();

        assert_eq!(string, "Hello");
        assert_eq!(buffer, b" World");
    }

    #[test]
    fn test_parse_over() {
        let mut buffer = b"Hello".as_slice();

        assert_eq!(
            ArrayString::<8>::parse(&mut buffer),
            Err(Error::Read(TryGetError {
                requested: 8,
                available: 5,
            }))
        );
        assert_eq!(buffer, []);
    }

    #[test]
    fn test_parse_invalid() {
        let mut buffer = [0x68, 0x80].as_slice();
        let err = ArrayString::<2>::parse(&mut buffer).unwrap_err();

        assert_matches!(err, Error::Utf8(err) if err.valid_up_to() == 1);
        assert_eq!(buffer, []);
    }
}
