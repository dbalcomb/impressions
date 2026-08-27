//! A region of unidentified bytes.

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::Extent;

/// A region of unidentified bytes.
///
/// This represents a region of memory that has not yet been identified. It may
/// contain initialised memory, uninitialised memory, or both depending on the
/// size of the region and the size of the internal bytes.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unidentified {
    size: u64,
    bytes: Bytes,
}

impl Unidentified {
    /// Constructs a new unidentified region.
    pub fn new(size: u64, mut bytes: Bytes) -> Self {
        bytes.truncate(size as usize);

        Self { size, bytes }
    }
}

impl Extent for Unidentified {
    fn size(&self) -> u64 {
        self.size
    }
}

impl Completion for Unidentified {
    fn identified(&self) -> u64 {
        0
    }
}

impl Debug for Unidentified {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = fmt::from_fn(|f| {
            let split = self.bytes.len() > 8;
            let prefix = self.bytes.iter().take(if split { 4 } else { 8 });

            for (index, byte) in prefix.enumerate() {
                if index > 0 {
                    write!(f, " ")?;
                }

                write!(f, "{byte:02x}")?;
            }

            if split {
                write!(f, " ...")?;

                for byte in self.bytes.iter().rev().take(4).rev() {
                    write!(f, " {byte:02x}")?;
                }
            }

            write!(f, " ({})", self.bytes.len())?;

            Ok(())
        });

        f.debug_struct("Unidentified")
            .field("size", &self.size)
            .field("bytes", &bytes)
            .finish()
    }
}
