//! A region of uninitialized memory.

mod error;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::{Extent, Slice, SliceBoundsError};

pub use self::error::Error;

/// A region of uninitialized memory.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Uninitialized(u64);

impl Uninitialized {
    /// Constructs a new uninitialized region.
    pub const fn new(size: u64) -> Result<Self, Error> {
        if size > u32::MAX as u64 + 1 {
            return Err(Error::SizeTooLarge(size));
        }

        Ok(Self(size))
    }

    /// Constructs a new empty uninitialized region.
    pub const fn empty() -> Self {
        Self(0)
    }
}

impl Slice for Uninitialized {
    type Error = Error;

    fn slice(&self, address: Address, size: u64) -> Result<Self, Self::Error> {
        let offset = address.value() as u64;
        let region_size = self.size();

        if offset >= region_size || size > region_size - offset {
            return Err(Error::SliceBounds(SliceBoundsError {
                address,
                size,
                region_size,
            }));
        }

        Self::new(size)
    }
}

impl Extent for Uninitialized {
    fn size(&self) -> u64 {
        self.0
    }
}

impl Completion for Uninitialized {
    fn identified(&self) -> u64 {
        0
    }
}

impl<'de> Deserialize<'de> for Uninitialized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u64::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::Address;
    use crate::memory::{Slice, SliceBoundsError};

    use super::{Error, Uninitialized};

    #[test]
    fn size_valid() {
        assert_eq!(Uninitialized::new(0), Ok(Uninitialized(0)));
        assert_eq!(Uninitialized::new(1), Ok(Uninitialized(1)));
        assert_eq!(
            Uninitialized::new(u32::MAX as u64),
            Ok(Uninitialized(u32::MAX as u64))
        );
        assert_eq!(
            Uninitialized::new(u32::MAX as u64 + 1),
            Ok(Uninitialized(u32::MAX as u64 + 1))
        );
    }

    #[test]
    fn size_invalid() {
        assert_eq!(
            Uninitialized::new(u32::MAX as u64 + 2),
            Err(Error::SizeTooLarge(u32::MAX as u64 + 2))
        );
        assert_eq!(
            Uninitialized::new(u64::MAX),
            Err(Error::SizeTooLarge(u64::MAX))
        );
    }

    #[test]
    fn slice_returns_requested_uninitialized_size() {
        let region = Uninitialized::new(10).unwrap();

        assert_eq!(
            region.slice(Address::new(0), 10),
            Ok(Uninitialized::new(10).unwrap())
        );
        assert_eq!(
            region.slice(Address::new(3), 4),
            Ok(Uninitialized::new(4).unwrap())
        );
        assert_eq!(
            region.slice(Address::new(9), 1),
            Ok(Uninitialized::new(1).unwrap())
        );
    }

    #[test]
    fn slice_allows_empty_slice_at_addressable_offset() {
        let region = Uninitialized::new(10).unwrap();

        assert_eq!(region.slice(Address::new(0), 0), Ok(Uninitialized::empty()));
        assert_eq!(region.slice(Address::new(5), 0), Ok(Uninitialized::empty()));
        assert_eq!(region.slice(Address::new(9), 0), Ok(Uninitialized::empty()));
    }

    #[test]
    fn slice_rejects_address_at_exclusive_end() {
        let region = Uninitialized::new(10).unwrap();

        assert_eq!(
            region.slice(Address::new(10), 0),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(10),
                size: 0,
                region_size: 10,
            })),
        );
    }

    #[test]
    fn slice_rejects_address_past_end() {
        let region = Uninitialized::new(10).unwrap();

        assert_eq!(
            region.slice(Address::new(11), 0),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(11),
                size: 0,
                region_size: 10,
            })),
        );
    }

    #[test]
    fn slice_rejects_size_past_end() {
        let region = Uninitialized::new(10).unwrap();

        assert_eq!(
            region.slice(Address::new(8), 3),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(8),
                size: 3,
                region_size: 10,
            })),
        );
    }

    #[test]
    fn slice_supports_full_address_space() {
        let size = u32::MAX as u64 + 1;
        let region = Uninitialized::new(size).unwrap();

        assert_eq!(
            region.slice(Address::new(u32::MAX), 1),
            Ok(Uninitialized::new(1).unwrap()),
        );

        assert_eq!(
            region.slice(Address::new(u32::MAX), 2),
            Err(Error::SliceBounds(SliceBoundsError {
                address: Address::new(u32::MAX),
                size: 2,
                region_size: size,
            })),
        );
    }
}
