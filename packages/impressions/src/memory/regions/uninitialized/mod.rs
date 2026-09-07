//! A region of uninitialized memory.

mod error;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::{Slice, SliceBoundsError};

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

    fn slice(&self, address: Address, size: Size) -> Result<Self, Self::Error> {
        let offset = address.value() as u64;
        let region_size = self.size();

        if offset >= region_size.get() || size.get() > region_size.get() - offset {
            return Err(Error::SliceBounds(SliceBoundsError {
                address,
                size,
                region_size,
            }));
        }

        Ok(Self::new(size))
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

impl Debug for Uninitialized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Uninitialized")
            .field("size", &self.0)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::Address;
    use crate::memory::extent::Size;
    use crate::memory::{Slice, SliceBoundsError};

    use super::{Error, Uninitialized};

    fn size(size: u64) -> super::Size {
        Size::new(size).unwrap()
    }

    #[test]
    fn slice_returns_requested_uninitialized_size() {
        let region = Uninitialized::new(size(10));

        assert_eq!(
            region.slice(Address::new(0), size(10)),
            Ok(Uninitialized::new(size(10)))
        );
        assert_eq!(
            region.slice(Address::new(3), size(4)),
            Ok(Uninitialized::new(size(4)))
        );
        assert_eq!(
            region.slice(Address::new(9), size(1)),
            Ok(Uninitialized::new(size(1)))
        );
    }

    #[test]
    fn slice_rejects_address_at_exclusive_end() {
        let region = Uninitialized::new(size(10));

        assert_eq!(
            region.slice(Address::new(10), size(1)),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(10),
                size: size(1),
                region_size: size(10),
            })),
        );
    }

    #[test]
    fn slice_rejects_address_past_end() {
        let region = Uninitialized::new(size(10));

        assert_eq!(
            region.slice(Address::new(11), size(1)),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(11),
                size: size(1),
                region_size: size(10),
            })),
        );
    }

    #[test]
    fn slice_rejects_size_past_end() {
        let region = Uninitialized::new(size(10));

        assert_eq!(
            region.slice(Address::new(8), size(3)),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(8),
                size: size(3),
                region_size: size(10),
            })),
        );
    }

    #[test]
    fn slice_supports_full_address_space() {
        let region = Uninitialized::new(size(u32::MAX as u64 + 1));

        assert_eq!(
            region.slice(Address::new(u32::MAX), size(1)),
            Ok(Uninitialized::new(size(1))),
        );

        assert_eq!(
            region.slice(Address::new(u32::MAX), size(2)),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(u32::MAX),
                size: size(2),
                region_size: size(u32::MAX as u64 + 1),
            })),
        );
    }
}
