use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::region::Region;

/// A block of padding.
///
/// This represents a block of bytes that has been identified as padding. This
/// may be found between sections, code, or data. In addition to an address and
/// size, each block of padding has a byte value to indicate what values have
/// been analysed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Padding {
    size: u64,
    value: u8,
}

impl Padding {
    /// Constructs a new padding block.
    pub fn new(size: u64, value: u8) -> Self {
        Self { size, value }
    }
}

impl Padding {
    /// Gets the padding value.
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Region for Padding {
    fn size(&self) -> u64 {
        self.size
    }
}

impl Completion for Padding {
    fn identified(&self) -> u64 {
        self.size()
    }
}
