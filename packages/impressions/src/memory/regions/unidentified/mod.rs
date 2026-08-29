//! A region of unidentified bytes.

mod error;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::Extent;

use super::initialized::Initialized;
use super::uninitialized::Uninitialized;

pub use self::error::Error;

/// A region of unidentified bytes.
///
/// This represents a region of memory that has not yet been identified. It may
/// contain initialised memory, uninitialised memory, or both depending on the
/// size of the region and the size of the internal bytes.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct Unidentified {
    initialized: Initialized,
    uninitialized: Uninitialized,
}

impl Unidentified {
    /// Constructs a new unidentified region.
    pub fn new(bytes: Bytes, uninitialized: u64) -> Result<Self, Error> {
        let initialized = Initialized::new(bytes)?;
        let uninitialized = Uninitialized::new(uninitialized)?;
        let size = initialized.size() + uninitialized.size();

        if size == 0 {
            return Err(Error::Empty);
        }

        if size > u32::MAX as u64 + 1 {
            return Err(Error::SizeTooLarge(size));
        }

        Ok(Self {
            initialized,
            uninitialized,
        })
    }
}

impl Unidentified {
    /// Gets the initialized region.
    pub fn initialized(&self) -> &Initialized {
        &self.initialized
    }

    /// Gets the uninitialized region.
    pub fn uninitialized(&self) -> &Uninitialized {
        &self.uninitialized
    }
}

impl Extent for Unidentified {
    fn size(&self) -> u64 {
        self.initialized.size() + self.uninitialized.size()
    }
}

impl Completion for Unidentified {
    fn identified(&self) -> u64 {
        0
    }
}

impl Debug for Unidentified {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unidentified")
            .field("initialized", &self.initialized())
            .field("uninitialized", &self.uninitialized.size())
            .field("size", &self.size())
            .finish()
    }
}

impl<'de> Deserialize<'de> for Unidentified {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Unidentified {
            initialized: Initialized,
            uninitialized: Uninitialized,
        }

        let this = Unidentified::deserialize(deserializer)?;

        Self::new(this.initialized.into(), this.uninitialized.size()).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::memory::Extent;

    use super::{Error, Unidentified};

    #[test]
    fn size_valid() {
        let bytes = Bytes::from_static(&[0, 1]);

        assert_eq!(Unidentified::new(bytes.clone(), 0).unwrap().size(), 2);
        assert_eq!(
            Unidentified::new(bytes.clone(), u32::MAX as u64 - 1)
                .unwrap()
                .size(),
            u32::MAX as u64 + 1
        );
    }

    #[test]
    fn size_invalid() {
        let bytes = Bytes::from_static(&[0, 1]);

        assert_eq!(Unidentified::new(Bytes::new(), 0), Err(Error::Empty));
        assert_eq!(
            Unidentified::new(bytes.clone(), u32::MAX as u64),
            Err(Error::SizeTooLarge(u32::MAX as u64 + 2))
        );
        assert_eq!(
            Unidentified::new(bytes.clone(), u32::MAX as u64 + 1),
            Err(Error::SizeTooLarge(u32::MAX as u64 + 3))
        );
    }
}
