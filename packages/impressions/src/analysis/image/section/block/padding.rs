use crate::memory::address::AddressSpace;
use crate::memory::region::Region;

/// A block of padding.
///
/// This represents a block of bytes that has been identified as padding. This
/// may be found between sections, code, or data. In addition to an address and
/// size, each block of padding has a byte value to indicate what values have
/// been analysed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Padding {
    address_space: AddressSpace,
    value: u8,
}

impl Padding {
    /// Constructs a new padding block.
    pub fn new(address_space: AddressSpace, value: u8) -> Self {
        Self {
            address_space,
            value,
        }
    }
}

impl Padding {
    /// Gets the padding value.
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Region for Padding {
    fn address_space(&self) -> AddressSpace {
        self.address_space
    }
}
