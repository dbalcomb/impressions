//! A region of initialized memory.

mod error;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::{Extent, Slice, SliceBoundsError};

pub use self::error::Error;

/// A region of initialized memory.
#[derive(Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Initialized(Bytes);

impl Initialized {
    /// Constructs a new initialized memory region.
    pub fn new(bytes: Bytes) -> Result<Self, Error> {
        let size = bytes.len() as u64;

        if size > u32::MAX as u64 + 1 {
            return Err(Error::SizeTooLarge(size));
        }

        Ok(Self(bytes))
    }

    /// Constructs an empty initialized memory region.
    pub const fn empty() -> Self {
        Self(Bytes::new())
    }
}

impl Initialized {
    /// Gets the bytes of the initialized memory region.
    pub const fn bytes(&self) -> &Bytes {
        &self.0
    }
}

impl Slice for Initialized {
    type Error = Error;

    fn slice(&self, offset: u32, size: u64) -> Result<Self, Self::Error> {
        let offset = offset as u64;
        let region_size = self.size();

        if offset >= region_size || size > region_size - offset {
            return Err(Error::SliceBounds(SliceBoundsError {
                offset: offset as u32,
                size,
                region_size,
            }));
        }

        let end = offset + size;

        Ok(Self(self.0.slice(offset as usize..end as usize)))
    }
}

impl Extent for Initialized {
    fn size(&self) -> u64 {
        self.0.len() as u64
    }
}

impl Completion for Initialized {
    fn identified(&self) -> u64 {
        0
    }
}

impl Debug for Initialized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let split = self.0.len() > 8;
        let prefix = self.0.iter().take(if split { 4 } else { 8 });

        write!(f, "[")?;

        for (index, byte) in prefix.enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }

            write!(f, "{byte:02x}")?;
        }

        if split {
            write!(f, " ...")?;

            for byte in self.0.iter().rev().take(4).rev() {
                write!(f, " {byte:02x}")?;
            }
        }

        write!(f, "] ({})", self.0.len())
    }
}

impl<'de> Deserialize<'de> for Initialized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(Bytes::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl TryFrom<Bytes> for Initialized {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::new(bytes)
    }
}

impl From<Initialized> for Bytes {
    fn from(region: Initialized) -> Self {
        region.0
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::memory::{Extent, Slice, SliceBoundsError};

    use super::{Error, Initialized};

    #[test]
    fn new_preserves_input_bytes() {
        let bytes = Bytes::from_static(b"hello");
        let region = Initialized::new(bytes).unwrap();

        assert_eq!(region.bytes(), "hello");
        assert_eq!(region.size(), 5);
    }

    #[test]
    fn new_allows_empty_bytes() {
        let region = Initialized::new(Bytes::new()).unwrap();

        assert_eq!(region, Initialized::empty());
        assert_eq!(region.size(), 0);
    }

    #[test]
    fn empty_has_no_bytes_or_extent() {
        let region = Initialized::empty();

        assert_eq!(region.bytes(), "");
        assert_eq!(region.size(), 0);
    }

    #[test]
    fn slice_returns_requested_bytes() {
        let region = Initialized::new(Bytes::from_static(b"abcdefghij")).unwrap();
        let slice = region.slice(3, 4).unwrap();

        assert_eq!(slice.bytes(), "defg");
        assert_eq!(slice.size(), 4);
    }

    #[test]
    fn slice_allows_empty_slice_at_addressable_offset() {
        let region = Initialized::new(Bytes::from_static(b"abcd")).unwrap();

        assert_eq!(region.slice(2, 0), Ok(Initialized::empty()));
    }

    #[test]
    fn slice_rejects_offset_at_exclusive_end() {
        let region = Initialized::new(Bytes::from_static(b"abcd")).unwrap();

        assert_eq!(
            region.slice(4, 0),
            Err(Error::SliceBounds(SliceBoundsError {
                offset: 4,
                size: 0,
                region_size: 4,
            })),
        );
    }

    #[test]
    fn slice_rejects_slice_past_end() {
        let region = Initialized::new(Bytes::from_static(b"abcd")).unwrap();

        assert_eq!(
            region.slice(3, 2),
            Err(Error::SliceBounds(SliceBoundsError {
                offset: 3,
                size: 2,
                region_size: 4,
            })),
        );
    }
}
