//! A region of initialized memory.

mod error;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::AddressSpace;
use crate::memory::cursor::{AsCursor, SimpleCursor};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};
use crate::memory::slice::{Error as SliceError, Slice};

pub use self::error::Error;

/// A region of initialized memory.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Initialized(Bytes);

impl Initialized {
    /// Constructs a new initialized memory region.
    pub fn new(bytes: Bytes) -> Result<Self, Error> {
        Size::new(bytes.len() as u64)?;

        Ok(Self(bytes))
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

    fn slice(&self, address_space: AddressSpace) -> Result<Self, Self::Error> {
        let region_address_space = self.address_space();

        if !region_address_space.includes(address_space) {
            return Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region_address_space,
            )));
        }

        Self::new(self.0.slice(address_space.to_index_range()))
    }
}

impl Extent for Initialized {
    fn size(&self) -> Size {
        Size::new(self.0.len() as u64).expect("valid size")
    }
}

impl Completion for Initialized {
    fn identified(&self) -> u64 {
        0
    }
}

impl Inspect for Initialized {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let mut output = inspector.unidentified();

        let split = self.0.len() > 8;
        let prefix = self.0.iter().take(if split { 4 } else { 8 });

        for (index, byte) in prefix.enumerate() {
            if index > 0 {
                write!(output, " ")?;
            }

            write!(output, "{byte:02x}")?;
        }

        if split {
            write!(output, " ...")?;

            for byte in self.0.iter().rev().take(4).rev() {
                write!(output, " {byte:02x}")?;
            }
        }

        writeln!(output)
    }
}

impl Debug for Initialized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = std::fmt::from_fn(|f| {
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

            write!(f, "]")
        });

        f.debug_struct("Initialized")
            .field("size", &self.0.len())
            .field("bytes", &bytes)
            .finish()
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

impl AsCursor for Initialized {
    #[rustfmt::skip]
    type Cursor<'a> = SimpleCursor<'a, Self>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        SimpleCursor::new(self)
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

    use crate::memory::address::Address;
    use crate::memory::extent::{Error as SizeError, Extent, Size};
    use crate::memory::slice::{Error as SliceError, Slice};

    use super::{Error, Initialized};

    #[test]
    fn new_preserves_input_bytes() {
        let bytes = Bytes::from_static(b"hello");
        let region = Initialized::new(bytes).unwrap();

        assert_eq!(region.bytes(), "hello");
        assert_eq!(region.size(), 5);
    }

    #[test]
    fn new_rejects_empty_bytes() {
        assert_eq!(
            Initialized::new(Bytes::new()),
            Err(Error::Size(SizeError::Zero))
        );
    }

    #[test]
    fn slice_returns_requested_bytes() {
        let region = Initialized::new(Bytes::from_static(b"abcdefghij")).unwrap();
        let slice = region
            .slice(Address::new(3).to_space(Size::new(4).unwrap()).unwrap())
            .unwrap();

        assert_eq!(slice.bytes(), "defg");
        assert_eq!(slice.size(), 4);
    }

    #[test]
    fn slice_rejects_address_at_exclusive_end() {
        let region = Initialized::new(Bytes::from_static(b"abcd")).unwrap();
        let address_space = Address::new(4).to_space(Size::new(1).unwrap()).unwrap();

        assert_eq!(
            region.slice(address_space),
            Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region.address_space(),
            ))),
        );
    }

    #[test]
    fn slice_rejects_slice_past_end() {
        let region = Initialized::new(Bytes::from_static(b"abcd")).unwrap();
        let address_space = Address::new(3).to_space(Size::new(2).unwrap()).unwrap();

        assert_eq!(
            region.slice(address_space),
            Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region.address_space(),
            ))),
        );
    }
}
