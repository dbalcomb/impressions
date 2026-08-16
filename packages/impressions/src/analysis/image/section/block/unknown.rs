use std::fmt::{self, Debug};

use bytes::Bytes;

/// A block of unknown bytes.
///
/// This represents a block of memory that has not yet been identified. It may
/// contain initialised memory, uninitialised memory, or both depending on the
/// size of the block and the size of the internal bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct Unknown {
    address: u32,
    size: u64,
    bytes: Bytes,
}

impl Unknown {
    /// Constructs a new unknown block.
    pub fn new(address: u32, size: u64, mut bytes: Bytes) -> Self {
        bytes.truncate(size as usize);

        Self {
            address,
            size,
            bytes,
        }
    }
}

impl Unknown {
    /// Gets the address of the unknown block.
    pub fn address(&self) -> u32 {
        self.address
    }

    /// Gets the size of the unknown block.
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Debug for Unknown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unknown")
            .field("address", &self.address)
            .field("size", &self.size)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}
