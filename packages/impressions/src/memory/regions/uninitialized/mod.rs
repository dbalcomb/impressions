//! A region of uninitialized memory.

mod error;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::Extent;

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
}
