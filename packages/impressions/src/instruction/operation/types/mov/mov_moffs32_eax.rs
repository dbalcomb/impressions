use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::prefix::Prefixes;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Move EAX to a dword at a direct 32-bit memory offset.
///
/// `A3 moffs32 => mov moffs32, eax`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovMoffs32Eax {
    offset: Address,
}

impl MovMoffs32Eax {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(offset: Address) -> Self {
        Self { offset }
    }
}

impl MovMoffs32Eax {
    /// Gets the offset operand.
    pub const fn offset(&self) -> Address {
        self.offset
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Mov
    }
}

impl MovMoffs32Eax {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} ", self.mnemonic())?;

        if prefixes.has_fs_override() {
            f.write_str("fs:")?;
        }

        write!(f, "[{}], eax", self.offset)
    }
}

impl Extent for MovMoffs32Eax {
    fn size(&self) -> Size {
        Size::new_valid(5)
    }
}

impl Encode for MovMoffs32Eax {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0xa3)?;
        self.offset.encode(encoder)
    }
}

impl Display for MovMoffs32Eax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operation::{Error, Operation};
    use crate::instruction::prefix::Prefixes;
    use crate::memory::address::Address;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn supports_direct_offsets() {
        for (bytes, offset, display) in [
            (
                &b"\xa3\x34\xbf\x60\x01"[..],
                0x0160bf34,
                "mov [0x0160bf34], eax",
            ),
            (&b"\xa3\x00\x00\x00\x00"[..], 0, "mov [0x00000000], eax"),
            (
                &b"\xa3\x78\x56\x34\x12"[..],
                0x12345678,
                "mov [0x12345678], eax",
            ),
            (
                &b"\xa3\x00\x00\x00\x80"[..],
                0x80000000,
                "mov [0x80000000], eax",
            ),
            (
                &b"\xa3\xff\xff\xff\xff"[..],
                u32::MAX,
                "mov [0xffffffff], eax",
            ),
        ] {
            let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
            let Operation::MovMoffs32Eax(operation) = operation else {
                panic!("expected MOV");
            };

            assert_eq!(operation.offset(), Address::new(offset));
            assert_eq!(operation.mnemonic(), Mnemonic::Mov);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), 5);

            let mut encoded = Vec::new();
            operation.encode(&mut encoded).unwrap();
            assert_eq!(encoded, bytes);
        }
    }

    #[test]
    fn rejects_truncation() {
        let bytes = b"\xa3\x34\xbf\x60\x01";
        for length in 1..bytes.len() {
            assert!(matches!(
                Operation::decode_with(&mut &bytes[..length], &Prefixes::new()),
                Err(Error::Decode(_)),
            ));
        }
    }
}
