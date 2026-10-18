use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::prefix::Prefixes;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Move a dword at a direct 32-bit memory offset to EAX.
///
/// `A1 moffs32 => mov eax, moffs32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovEaxMoffs32 {
    offset: Address,
}

impl MovEaxMoffs32 {
    /// Creates the operation from its encoded operands.
    pub const fn new(offset: Address) -> Self {
        Self { offset }
    }

    /// Gets the offset operand.
    pub const fn offset(&self) -> Address {
        self.offset
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Mov
    }
}

impl MovEaxMoffs32 {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} eax, ", self.mnemonic())?;

        if prefixes.has_fs_override() {
            f.write_str("fs:")?;
        }

        write!(f, "[{}]", self.offset)
    }
}

impl Extent for MovEaxMoffs32 {
    fn size(&self) -> Size {
        Size::new_valid(5)
    }
}

impl Encode for MovEaxMoffs32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0xa1)?;
        self.offset.encode(encoder)
    }
}

impl Display for MovEaxMoffs32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operation::Operation;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::address::Address;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    use super::MovEaxMoffs32;

    #[test]
    fn supports_direct_offsets() {
        for (bytes, offset, display) in [
            (&b"\xa1\x00\x00\x00\x00"[..], 0, "mov eax, [0x00000000]"),
            (
                &b"\xa1\x78\x56\x34\x12"[..],
                0x12345678,
                "mov eax, [0x12345678]",
            ),
            (
                &b"\xa1\xff\xff\xff\xff"[..],
                u32::MAX,
                "mov eax, [0xffffffff]",
            ),
        ] {
            let mut decoder = bytes;
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoded = Vec::new();

            operation.encode(&mut encoded).unwrap();

            assert_eq!(
                operation,
                Operation::MovEaxMoffs32(MovEaxMoffs32::new(Address::new(offset)))
            );
            assert_eq!(operation.mnemonic(), Mnemonic::Mov);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), 5);
            assert_eq!(encoded, bytes);
            assert!(decoder.is_empty());
        }
    }
}
