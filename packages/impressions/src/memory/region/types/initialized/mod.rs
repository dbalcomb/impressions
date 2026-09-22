//! A region of initialized memory.

mod cursor;
mod error;

use std::fmt::{self, Debug, Display};

use bytes::Bytes;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::AddressSpace;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, InspectionValue, Inspector};
use crate::memory::region::ops::encode::{self, Encode};
use crate::memory::region::ops::slice::{Error as SliceError, Slice};

pub use self::cursor::InitializedCursor;
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
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .unidentified()
            .label(&"Initialized")
            .value(self)
            .finish()
    }
}

impl Encode for Initialized {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        encoder.write(self.0.as_ref())
    }
}

impl InspectionValue for Initialized {
    fn data_type(&self) -> &dyn Display {
        &"bytes"
    }
}

impl Display for Initialized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let split = self.0.len() > 8;
        let prefix = self.0.iter().take(if split { 4 } else { 8 });

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

        Ok(())
    }
}

impl Debug for Initialized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Initialized")
            .field("size", &self.0.len())
            .field("bytes", &format_args!("[{}]", self))
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
    type Cursor<'a> = InitializedCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        InitializedCursor::new(self)
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
    use crate::memory::region::ops::slice::{Error as SliceError, Slice};

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
