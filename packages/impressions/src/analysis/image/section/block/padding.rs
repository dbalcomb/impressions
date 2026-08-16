/// A block of padding.
///
/// This represents a block of bytes that has been identified as padding. This
/// may be found between sections, code, or data. In addition to an address and
/// size, each block of padding has a byte value to indicate what values have
/// been analysed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Padding {
    address: u32,
    size: u64,
    value: u8,
}

impl Padding {
    /// Constructs a new padding block.
    pub fn new(address: u32, size: u64, value: u8) -> Self {
        Self {
            address,
            size,
            value,
        }
    }
}

impl Padding {
    /// Gets the address of the padding block.
    pub fn address(&self) -> u32 {
        self.address
    }

    /// Gets the size of the padding block.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Gets the padding value.
    pub fn value(&self) -> u8 {
        self.value
    }
}
