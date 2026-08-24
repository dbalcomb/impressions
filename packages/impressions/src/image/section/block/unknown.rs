use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::region::Region;

/// A block of unknown bytes.
///
/// This represents a block of memory that has not yet been identified. It may
/// contain initialised memory, uninitialised memory, or both depending on the
/// size of the block and the size of the internal bytes.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unknown {
    size: u64,
    bytes: Bytes,
}

impl Unknown {
    /// Constructs a new unknown block.
    pub fn new(size: u64, mut bytes: Bytes) -> Self {
        bytes.truncate(size as usize);

        Self { size, bytes }
    }
}

impl Region for Unknown {
    fn size(&self) -> u64 {
        self.size
    }
}

impl Completion for Unknown {
    fn identified(&self) -> u64 {
        0
    }
}

impl Debug for Unknown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unknown")
            .field("size", &self.size)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}
