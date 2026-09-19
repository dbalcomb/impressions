//! The null-terminated string data type.

mod error;

use std::fmt::{self, Debug, Display};
use std::ops::Deref;

use bytes::Buf;
use serde::{Deserialize, Deserializer, Serialize};

use crate::data::parse::Parse;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::InspectionValue;

pub use self::error::Error;

/// A UTF-8 string terminated by a null byte.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct NullString(String);

impl NullString {
    /// Gets the length of the string.
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks whether the string is empty.
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Extent for NullString {
    fn size(&self) -> Size {
        Size::new(self.0.len() as u64 + 1).expect("valid size")
    }
}

impl InspectionValue for NullString {
    fn data_type(&self) -> &dyn Display {
        &"string"
    }
}

impl Parse for NullString {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let mut bytes = Vec::new();

        while buffer.has_remaining() {
            let chunk = buffer.chunk();

            if let Some(index) = chunk.iter().position(|&byte| byte == 0) {
                bytes.extend_from_slice(&chunk[..index]);
                buffer.advance(index + 1);

                let value =
                    String::from_utf8(bytes).map_err(|err| Error::Utf8(err.utf8_error()))?;

                return Ok(Self(value));
            }

            bytes.extend_from_slice(chunk);
            buffer.advance(chunk.len());
        }

        Err(Error::MissingNull)
    }
}

impl PartialEq<str> for NullString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for NullString {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Debug for NullString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for NullString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for NullString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for NullString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for NullString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};

        struct NullStringVisitor;

        impl<'de> Visitor<'de> for NullStringVisitor {
            type Value = NullString;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string without null bytes")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if value.contains('\0') {
                    return Err(E::invalid_value(Unexpected::Char('\0'), &self));
                }

                Ok(NullString(value.to_owned()))
            }
        }

        deserializer.deserialize_str(NullStringVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crate::data::parse::Parse;

    use super::{Error, NullString};

    #[test]
    fn parse() {
        let mut buffer = b"Hello\0World".as_slice();
        let string = NullString::parse(&mut buffer).unwrap();

        assert_eq!(string, "Hello");
        assert_eq!(buffer, b"World");
    }

    #[test]
    fn parse_empty() {
        let mut buffer = b"\0".as_slice();
        let string = NullString::parse(&mut buffer).unwrap();

        assert!(string.is_empty());
        assert!(buffer.is_empty());
    }

    #[test]
    fn parse_missing_null() {
        let mut buffer = b"Hello".as_slice();

        assert_eq!(NullString::parse(&mut buffer), Err(Error::MissingNull));
        assert!(buffer.is_empty());
    }

    #[test]
    fn parse_invalid_utf8() {
        let mut buffer = [b'H', 0x80, 0].as_slice();
        let err = NullString::parse(&mut buffer).unwrap_err();

        assert_matches!(err, Error::Utf8(err) if err.valid_up_to() == 1);
        assert!(buffer.is_empty());
    }

    #[test]
    fn parse_serde() {
        let string = NullString::parse(b"hello\0".as_slice()).unwrap();
        let json = serde_json::to_string(&string).unwrap();

        assert_eq!(json, "\"hello\"");
        assert_eq!(serde_json::from_str::<NullString>(&json).unwrap(), string);
        assert!(
            serde_json::from_str::<NullString>("\"hello\\u0000world\"")
                .unwrap_err()
                .is_data()
        );
    }
}
