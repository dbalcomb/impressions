use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::{Extent, Slice, SliceBoundsError};

/// A region of padding.
///
/// This represents a region of bytes that has been identified as padding. This
/// may be found between sections, code, or data. Each region of padding has a
/// byte value to indicate what values have been analysed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Padding {
    size: u64,
    value: u8,
}

impl Padding {
    /// Constructs a new padding region.
    pub const fn new(size: u64, value: u8) -> Self {
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
    fn size(&self) -> u64 {
        self.size
    }
}

impl Completion for Padding {
    fn identified(&self) -> u64 {
        self.size
    }
}

impl Slice for Padding {
    type Error = SliceBoundsError;

    fn slice(&self, offset: u32, size: u64) -> Result<Self, Self::Error> {
        let offset = offset as u64;
        let region_size = self.size();

        if offset >= region_size || size > region_size - offset {
            return Err(SliceBoundsError {
                offset: offset as u32,
                size,
                region_size,
            });
        }

        Ok(Self::new(size, self.value()))
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{Extent, Slice, SliceBoundsError};

    use super::Padding;

    #[test]
    fn slice_preserves_padding_value() {
        let padding = Padding::new(10, 0xcc);

        let slice = padding.slice(3, 4).unwrap();

        assert_eq!(slice.size(), 4);
        assert_eq!(slice.value(), 0xcc);
    }

    #[test]
    fn slice_allows_empty_slice_at_addressable_offset() {
        let padding = Padding::new(10, 0xcc);

        assert_eq!(padding.slice(0, 0), Ok(Padding::new(0, 0xcc)));
        assert_eq!(padding.slice(5, 0), Ok(Padding::new(0, 0xcc)));
        assert_eq!(padding.slice(9, 0), Ok(Padding::new(0, 0xcc)));
    }

    #[test]
    fn slice_rejects_offset_at_exclusive_end() {
        let padding = Padding::new(10, 0xcc);

        assert_eq!(
            padding.slice(10, 0),
            Err(SliceBoundsError {
                offset: 10,
                size: 0,
                region_size: 10,
            }),
        );
    }

    #[test]
    fn slice_rejects_range_that_extends_past_end() {
        let padding = Padding::new(10, 0xcc);

        assert_eq!(
            padding.slice(8, 3),
            Err(SliceBoundsError {
                offset: 8,
                size: 3,
                region_size: 10,
            }),
        );
    }
}
