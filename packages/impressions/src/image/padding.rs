use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::{Extent, Size, Slice, SliceBoundsError};

/// A region of padding.
///
/// This represents a region of bytes that has been identified as padding. This
/// may be found between sections, code, or data. Each region of padding has a
/// byte value to indicate what values have been analysed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Padding {
    size: Size,
    value: u8,
}

impl Padding {
    /// Constructs a new padding region.
    pub const fn new(size: Size, value: u8) -> Self {
        Self { size, value }
    }
}

impl Padding {
    /// Gets the padding value.
    pub const fn value(&self) -> u8 {
        self.value
    }
}

impl Extent for Padding {
    fn size(&self) -> Size {
        self.size
    }
}

impl Completion for Padding {
    fn identified(&self) -> u64 {
        self.size.get()
    }
}

impl Slice for Padding {
    type Error = SliceBoundsError;

    fn slice(&self, address: Address, size: Size) -> Result<Self, Self::Error> {
        let offset = address.value() as u64;
        let region_size = self.size();

        if offset >= region_size.get() || size.get() > region_size.get() - offset {
            return Err(SliceBoundsError {
                address,
                size,
                region_size,
            });
        }

        Ok(Self::new(size, self.value()))
    }
}

impl Debug for Padding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Padding")
            .field("size", &self.size)
            .field("value", &format_args!("{:02x}", self.value))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::Address;
    use crate::memory::{Extent, Size, Slice, SliceBoundsError};

    use super::Padding;

    #[test]
    fn slice_preserves_padding_value() {
        let padding = Padding::new(Size::new(10).unwrap(), 0xcc);
        let slice = padding
            .slice(Address::new(3), Size::new(4).unwrap())
            .unwrap();

        assert_eq!(slice.size(), 4);
        assert_eq!(slice.value(), 0xcc);
    }

    #[test]
    fn slice_rejects_address_at_exclusive_end() {
        let padding = Padding::new(Size::new(10).unwrap(), 0xcc);

        assert_eq!(
            padding.slice(Address::new(10), Size::new(1).unwrap()),
            Err(SliceBoundsError {
                address: Address::new(10),
                size: Size::new(1).unwrap(),
                region_size: Size::new(10).unwrap(),
            }),
        );
    }

    #[test]
    fn slice_rejects_range_that_extends_past_end() {
        let padding = Padding::new(Size::new(10).unwrap(), 0xcc);

        assert_eq!(
            padding.slice(Address::new(8), Size::new(3).unwrap()),
            Err(SliceBoundsError {
                address: Address::new(8),
                size: Size::new(3).unwrap(),
                region_size: Size::new(10).unwrap(),
            }),
        );
    }
}
