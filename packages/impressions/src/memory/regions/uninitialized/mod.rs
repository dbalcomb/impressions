//! A region of uninitialized memory.

mod error;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::AddressSpace;
use crate::memory::cursor::{AsCursor, SimpleCursor};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};
use crate::memory::slice::{Error as SliceError, Slice};

pub use self::error::Error;

/// A region of uninitialized memory.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Uninitialized(Size);

impl Uninitialized {
    /// Constructs a new uninitialized region.
    pub const fn new(size: Size) -> Self {
        Self(size)
    }
}

impl Slice for Uninitialized {
    type Error = Error;

    fn slice(&self, address_space: AddressSpace) -> Result<Self, Self::Error> {
        let region_address_space = self.address_space();

        if !region_address_space.includes(address_space) {
            return Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region_address_space,
            )));
        }

        Ok(Self::new(address_space.size()))
    }
}

impl Extent for Uninitialized {
    fn size(&self) -> Size {
        self.0
    }
}

impl Completion for Uninitialized {
    fn identified(&self) -> u64 {
        0
    }
}

impl Inspect for Uninitialized {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        writeln!(inspector.unidentified(), "00 ...")
    }
}

impl Debug for Uninitialized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Uninitialized")
            .field("size", &self.0)
            .finish()
    }
}

impl AsCursor for Uninitialized {
    #[rustfmt::skip]
    type Cursor<'a> = SimpleCursor<'a, Self>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        SimpleCursor::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::{Address, AddressSpace};
    use crate::memory::extent::{Extent, Size};
    use crate::memory::slice::{Error as SliceError, Slice};

    use super::{Error, Uninitialized};

    fn size(size: u64) -> super::Size {
        Size::new(size).unwrap()
    }

    #[test]
    fn slice_returns_requested_uninitialized_size() {
        let region = Uninitialized::new(size(10));

        assert_eq!(
            region.slice(Address::new(0).to_space(size(10)).unwrap()),
            Ok(Uninitialized::new(size(10)))
        );
        assert_eq!(
            region.slice(Address::new(3).to_space(size(4)).unwrap()),
            Ok(Uninitialized::new(size(4)))
        );
        assert_eq!(
            region.slice(Address::new(9).to_space(size(1)).unwrap()),
            Ok(Uninitialized::new(size(1)))
        );
    }

    #[test]
    fn slice_rejects_address_at_exclusive_end() {
        let region = Uninitialized::new(size(10));
        let address_space = Address::new(10).to_space(size(1)).unwrap();

        assert_eq!(
            region.slice(address_space),
            Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region.address_space(),
            ))),
        );
    }

    #[test]
    fn slice_rejects_address_past_end() {
        let region = Uninitialized::new(size(10));
        let address_space = Address::new(11).to_space(size(1)).unwrap();

        assert_eq!(
            region.slice(address_space),
            Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region.address_space(),
            ))),
        );
    }

    #[test]
    fn slice_rejects_size_past_end() {
        let region = Uninitialized::new(size(10));
        let address_space = Address::new(8).to_space(size(3)).unwrap();

        assert_eq!(
            region.slice(address_space),
            Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region.address_space(),
            ))),
        );
    }

    #[test]
    fn slice_supports_full_address_space() {
        let region = Uninitialized::new(Size::MAX);

        assert_eq!(
            region.slice(Address::new(u32::MAX).to_space(size(1)).unwrap()),
            Ok(Uninitialized::new(size(1))),
        );

        assert_eq!(
            region.slice(AddressSpace::default()),
            Ok(Uninitialized::new(Size::MAX)),
        );
    }
}
