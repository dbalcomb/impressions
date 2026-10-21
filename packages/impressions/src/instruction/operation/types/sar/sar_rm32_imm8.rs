use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register32;
use crate::instruction::operation::encoding::Rm32;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Shift a 32-bit register or memory value right arithmetically by an unsigned
/// byte count.
///
/// `C1 /7 ib => sar r/m32, imm8`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SarRm32Imm8 {
    operand: Rm32,
    count: u8,
}

impl SarRm32Imm8 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(operand: Rm32, count: u8) -> Self {
        Self { operand, count }
    }
}

impl SarRm32Imm8 {
    /// Gets the destination register, if the destination is not memory.
    pub const fn destination_register(&self) -> Option<Register32> {
        self.operand.rm_register()
    }

    /// Gets the destination address, if the destination is memory.
    pub const fn destination_memory(&self) -> Option<Memory32> {
        self.operand.memory()
    }

    /// Gets the encoded count, before the CPU masks it to five bits.
    pub const fn count(&self) -> u8 {
        self.count
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Sar
    }
}

impl SarRm32Imm8 {
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

        write!(f, "{}, {}", self.operand, self.count)
    }
}

impl Extent for SarRm32Imm8 {
    fn size(&self) -> Size {
        Size::new_valid(2 + self.operand.size().get())
    }
}

impl Encode for SarRm32Imm8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0xc1)?;
        self.operand.encode(encoder)?;
        encoder.write_u8(self.count)
    }
}

impl Display for SarRm32Imm8 {
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
    fn supports_all_registers_and_byte_counts() {
        for register in 0..8 {
            for count in u8::MIN..=u8::MAX {
                let bytes = [0xc1, 0xf8 | register, count];
                let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                let Operation::SarRm32Imm8(operation) = operation else {
                    panic!("expected SAR");
                };
                let register = Register32::from_index(register);
                assert_eq!(operation.destination_register(), Some(register));
                assert_eq!(operation.destination_memory(), None);
                assert_eq!(operation.count(), count);
                assert_eq!(operation.mnemonic(), Mnemonic::Sar);
                assert_eq!(operation.to_string(), format!("sar {register}, {count}"));
                assert_eq!(operation.size(), 3);
                let mut encoded = Vec::new();
                operation.encode(&mut encoded).unwrap();
                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn supports_memory_forms() {
        for (operand_bytes, memory) in [
            (&b"\x3b"[..], "[ebx]"),
            (&b"\x7b\xf0"[..], "[ebx-16]"),
            (&b"\xbb\xf0\xff\xff\xff"[..], "[ebx-16]"),
            (&b"\x3d\x38\x81\x5d\x00"[..], "[0x005d8138]"),
            (&b"\x3c\x24"[..], "[esp]"),
            (&b"\x7d\x00"[..], "[ebp]"),
            (&b"\x3c\x8b"[..], "[ebx+ecx*4]"),
            (&b"\x7c\x8b\xf0"[..], "[ebx+ecx*4-16]"),
            (&b"\xbc\x8b\xf0\xff\xff\xff"[..], "[ebx+ecx*4-16]"),
            (&b"\x3c\x8d\x10\x00\x00\x00"[..], "[ecx*4+16]"),
            (&b"\x3c\xe5\x38\x81\x5d\x00"[..], "[0x005d8138]"),
        ] {
            let mut bytes = vec![0xc1];
            bytes.extend_from_slice(operand_bytes);
            bytes.push(8);
            let operation =
                Operation::decode_with(&mut bytes.as_slice(), &Prefixes::new()).unwrap();
            let Operation::SarRm32Imm8(operation) = operation else {
                panic!("expected SAR");
            };
            assert_eq!(operation.destination_register(), None);
            assert_eq!(operation.destination_memory().unwrap().to_string(), memory);
            assert_eq!(operation.count(), 8);
            assert_eq!(operation.to_string(), format!("sar dword ptr {memory}, 8"));
            assert_eq!(operation.size(), bytes.len() as u64);
            let mut encoded = Vec::new();
            operation.encode(&mut encoded).unwrap();
            assert_eq!(encoded, bytes);
        }
    }

    #[test]
    fn rejects_truncation() {
        for bytes in [
            &b"\xc1\xf9\x08"[..],
            &b"\xc1\x7c\x8b\xf0\x08"[..],
            &b"\xc1\x3d\x38\x81\x5d\x00\x08"[..],
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
