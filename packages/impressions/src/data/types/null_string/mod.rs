//! The null-terminated string data type.

mod error;

use std::fmt::{self, Debug, Display};
use std::ops::Deref;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::InspectionValue;

pub use self::error::Error;

/// A UTF-8 string terminated by a null byte.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
