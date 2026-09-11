use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::AddressSpace;
use crate::memory::cursor::{AsCursor, SimpleCursor};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};
use crate::memory::slice::{Error as SliceError, Slice};

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
    type Error = SliceError;

    fn slice(&self, address_space: AddressSpace) -> Result<Self, Self::Error> {
        let region_address_space = self.address_space();

        if !region_address_space.includes(address_space) {
            return Err(SliceError::OutOfBounds(address_space, region_address_space));
        }

        Ok(Self::new(address_space.size(), self.value()))
    }
}

impl Inspect for Padding {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let mut output = inspector.identified();

        writeln!(output, "Padding")?;

        output.nest();

        writeln!(output, "{:02x} ...", self.value())
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

impl AsCursor for Padding {
    type Cursor<'a> = SimpleCursor<'a, Self>;

    fn cursor(&self) -> Self::Cursor<'_> {
        SimpleCursor::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::Address;
    use crate::memory::extent::{Extent, Size};
    use crate::memory::slice::{Error as SliceError, Slice};

    use super::Padding;

    #[test]
    fn slice_preserves_padding_value() {
        let padding = Padding::new(Size::new(10).unwrap(), 0xcc);
        let slice = padding
            .slice(Address::new(3).to_space(Size::new(4).unwrap()).unwrap())
            .unwrap();

        assert_eq!(slice.size(), 4);
        assert_eq!(slice.value(), 0xcc);
    }

    #[test]
    fn slice_rejects_address_at_exclusive_end() {
        let padding = Padding::new(Size::new(10).unwrap(), 0xcc);
        let address_space = Address::new(10).to_space(Size::new(1).unwrap()).unwrap();

        assert_eq!(
            padding.slice(address_space),
            Err(SliceError::OutOfBounds(
                address_space,
                padding.address_space(),
            )),
        );
    }

    #[test]
    fn slice_rejects_range_that_extends_past_end() {
        let padding = Padding::new(Size::new(10).unwrap(), 0xcc);
        let address_space = Address::new(8).to_space(Size::new(3).unwrap()).unwrap();

        assert_eq!(
            padding.slice(address_space),
            Err(SliceError::OutOfBounds(
                address_space,
                padding.address_space(),
            )),
        );
    }
}
