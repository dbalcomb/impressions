use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::immediate::Immediate32;
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register32;
use crate::instruction::operation::encoding::Rm32;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Subtract a 32-bit immediate and borrow from r/m32.
///
/// `81 /3 id => sbb r/m32, imm32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SbbRm32Imm32 {
    operand: Rm32,
    immediate: Immediate32,
}

impl SbbRm32Imm32 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(
        operand: Rm32,
        immediate: Immediate32,
    ) -> Self {
        Self { operand, immediate }
    }
}

impl SbbRm32Imm32 {
    /// Gets the destination register, if the destination is not memory.
    pub const fn destination_register(&self) -> Option<Register32> {
        self.operand.rm_register()
    }

    /// Gets the destination address, if the destination is memory.
    pub const fn destination_memory(&self) -> Option<Memory32> {
        self.operand.memory()
    }

    /// Gets the signed dword immediate.
    pub const fn immediate(&self) -> Immediate32 {
        self.immediate
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Sbb
    }
}

impl SbbRm32Imm32 {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} ", self.mnemonic())?;

        if self.operand.memory().is_some() {
            f.write_str("dword ptr ")?;
        }

        if prefixes.has_fs_override() && self.operand.memory().is_some() {
            f.write_str("fs:")?;
        }

        write!(f, "{}, {}", self.operand, self.immediate)
    }
}

impl Extent for SbbRm32Imm32 {
    fn size(&self) -> Size {
        Size::new_valid(5 + self.operand.size().get())
    }
}

impl Encode for SbbRm32Imm32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x81)?;
        self.operand.encode(encoder)?;
        self.immediate.encode(encoder)
    }
}

impl Display for SbbRm32Imm32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::immediate::Immediate32;
    use crate::instruction::operand::register::Register32;
    use crate::instruction::operation::{Error, Operation};
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn supports_all_registers_and_dword_immediates() {
        for register in 0..8 {
            for value in [i32::MIN, -128, -1, 0, 1, 127, 255, 0x12345678, i32::MAX] {
                let mut bytes = vec![0x81, 0xd8 | register];
                bytes.extend_from_slice(&value.to_le_bytes());
                let operation =
                    Operation::decode_with(&mut bytes.as_slice(), &Prefixes::new()).unwrap();
                let Operation::SbbRm32Imm32(operation) = operation else {
                    panic!("expected SBB");
                };
                let register = Register32::from_index(register);

                assert_eq!(operation.destination_register(), Some(register));
                assert_eq!(operation.destination_memory(), None);
                assert_eq!(operation.immediate(), Immediate32::new(value));
                assert_eq!(operation.mnemonic(), Mnemonic::Sbb);
                assert_eq!(operation.to_string(), format!("sbb {register}, {value}"));
                assert_eq!(operation.size(), 6);

                let mut encoded = Vec::new();
                operation.encode(&mut encoded).unwrap();
                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn supports_memory_forms() {
        for (operand_bytes, memory) in [
            (&b"\x1b"[..], "[ebx]"),
            (&b"\x5b\xf0"[..], "[ebx-16]"),
            (&b"\x9b\xf0\xff\xff\xff"[..], "[ebx-16]"),
            (&b"\x1d\x38\x81\x5d\x00"[..], "[0x005d8138]"),
            (&b"\x1c\x24"[..], "[esp]"),
            (&b"\x5d\x00"[..], "[ebp]"),
            (&b"\x1c\x8b"[..], "[ebx+ecx*4]"),
            (&b"\x5c\x8b\xf0"[..], "[ebx+ecx*4-16]"),
            (&b"\x9c\x8b\xf0\xff\xff\xff"[..], "[ebx+ecx*4-16]"),
            (&b"\x1c\x8d\x10\x00\x00\x00"[..], "[ecx*4+16]"),
            (&b"\x1c\xe5\x38\x81\x5d\x00"[..], "[0x005d8138]"),
        ] {
            for value in [i32::MIN, -1, 0, 255, i32::MAX] {
                let mut bytes = vec![0x81];
                bytes.extend_from_slice(operand_bytes);
                bytes.extend_from_slice(&value.to_le_bytes());
                let operation =
                    Operation::decode_with(&mut bytes.as_slice(), &Prefixes::new()).unwrap();
                let Operation::SbbRm32Imm32(operation) = operation else {
                    panic!("expected SBB");
                };

                assert_eq!(operation.destination_register(), None);
                assert_eq!(operation.destination_memory().unwrap().to_string(), memory);
                assert_eq!(operation.immediate(), Immediate32::new(value));
                assert_eq!(
                    operation.to_string(),
                    format!("sbb dword ptr {memory}, {value}")
                );
                assert_eq!(operation.size(), bytes.len() as u64);

                let mut encoded = Vec::new();
                operation.encode(&mut encoded).unwrap();
                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn rejects_truncation() {
        for bytes in [
            &b"\x81\xd9\xff\x00\x00\x00"[..],
            &b"\x81\x5c\x8b\xf0\xff\x00\x00\x00"[..],
            &b"\x81\x1d\x38\x81\x5d\x00\xff\x00\x00\x00"[..],
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
