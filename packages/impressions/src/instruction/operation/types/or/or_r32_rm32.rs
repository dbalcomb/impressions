use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register32;
use crate::instruction::operation::encoding::Rm32;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Bitwise OR a 32-bit register with a 32-bit register or memory value.
///
/// `0B /r => or r32, r/m32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrR32Rm32 {
    operand: Rm32,
}

impl OrR32Rm32 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(operand: Rm32) -> Self {
        Self { operand }
    }
}

impl OrR32Rm32 {
    /// Gets the source register, if the source is not memory.
    pub const fn source_register(&self) -> Option<Register32> {
        self.operand.rm_register()
    }

    /// Gets the source address, if the source is memory.
    pub const fn source_memory(&self) -> Option<Memory32> {
        self.operand.memory()
    }

    /// Gets the destination register.
    pub const fn destination_register(&self) -> Register32 {
        self.operand.register()
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Or
    }
}

impl OrR32Rm32 {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} {}, ", self.mnemonic(), self.destination_register())?;

        if prefixes.has_fs_override() && self.operand.memory().is_some() {
            f.write_str("fs:")?;
        }

        Display::fmt(&self.operand, f)
    }
}

impl Extent for OrR32Rm32 {
    fn size(&self) -> Size {
        Size::new_valid(1 + self.operand.size().get())
    }
}

impl Encode for OrR32Rm32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x0b)?;
        self.operand.encode(encoder)
    }
}

impl Display for OrR32Rm32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::register::Register32;
    use crate::instruction::operation::{Error, Operation};
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn supports_all_register_forms() {
        for destination in 0..8 {
            for source in 0..8 {
                let bytes = [0x0b, 0xc0 | destination << 3 | source];
                let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                let Operation::OrR32Rm32(operation) = operation else {
                    panic!("expected OR");
                };
                let destination = Register32::from_index(destination);
                let source = Register32::from_index(source);

                assert_eq!(operation.destination_register(), destination);
                assert_eq!(operation.source_register(), Some(source));
                assert_eq!(operation.source_memory(), None);
                assert_eq!(operation.mnemonic(), Mnemonic::Or);
                assert_eq!(operation.to_string(), format!("or {destination}, {source}"));
                assert_eq!(operation.size(), 2);

                let mut encoded = Vec::new();

                operation.encode(&mut encoded).unwrap();

                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn supports_memory_forms() {
        for (bytes, memory) in [
            (&b"\x0b\x13"[..], "[ebx]"),
            (&b"\x0b\x53\xf0"[..], "[ebx-16]"),
            (&b"\x0b\x93\xf0\xff\xff\xff"[..], "[ebx-16]"),
            (&b"\x0b\x15\x38\x81\x5d\x00"[..], "[0x005d8138]"),
            (&b"\x0b\x14\x24"[..], "[esp]"),
            (&b"\x0b\x55\x00"[..], "[ebp]"),
            (&b"\x0b\x14\x8b"[..], "[ebx+ecx*4]"),
            (&b"\x0b\x54\x8b\xf0"[..], "[ebx+ecx*4-16]"),
            (&b"\x0b\x94\x8b\xf0\xff\xff\xff"[..], "[ebx+ecx*4-16]"),
            (&b"\x0b\x14\x8d\x10\x00\x00\x00"[..], "[ecx*4+16]"),
            (&b"\x0b\x14\xe5\x38\x81\x5d\x00"[..], "[0x005d8138]"),
            (&b"\x0b\x53\x00"[..], "[ebx]"),
            (&b"\x0b\x93\x00\x00\x00\x00"[..], "[ebx]"),
            (&b"\x0b\x14\x0b"[..], "[ebx+ecx*1]"),
            (&b"\x0b\x14\x4b"[..], "[ebx+ecx*2]"),
            (&b"\x0b\x14\xcb"[..], "[ebx+ecx*8]"),
        ] {
            let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
            let Operation::OrR32Rm32(operation) = operation else {
                panic!("expected OR");
            };

            assert_eq!(operation.destination_register(), Register32::Edx);
            assert_eq!(operation.source_register(), None);
            assert_eq!(operation.source_memory().unwrap().to_string(), memory);
            assert_eq!(operation.mnemonic(), Mnemonic::Or);
            assert_eq!(operation.to_string(), format!("or edx, {memory}"));
            assert_eq!(operation.size(), bytes.len() as u64);

            let mut encoded = Vec::new();

            operation.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }
    }

    #[test]
    fn rejects_truncation() {
        for bytes in [
            &b"\x0b\xd2"[..],
            &b"\x0b\x15\x38\x81\x5d\x00"[..],
            &b"\x0b\x94\x8b\xf0\xff\xff\xff"[..],
        ] {
            for length in 1..bytes.len() {
                assert!(matches!(
                    Operation::decode_with(&mut &bytes[..length], &Prefixes::new()),
                    Err(Error::Decode(_)),
                ));
            }
        }
    }
}
