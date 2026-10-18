use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::immediate::Immediate8;
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register32;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use crate::instruction::operation::encoding::Rm32;

/// Subtract a sign-extended byte immediate from r/m32.
///
/// `83 /5 ib => sub r/m32, imm8`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubRm32Imm8 {
    operand: Rm32,
    immediate: Immediate8,
}

impl SubRm32Imm8 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(
        operand: Rm32,
        immediate: Immediate8,
    ) -> Self {
        Self { operand, immediate }
    }
}

impl SubRm32Imm8 {
    /// Gets the destination register, if the destination is not memory.
    pub const fn destination_register(&self) -> Option<Register32> {
        self.operand.rm_register()
    }

    /// Gets the destination address, if the destination is memory.
    pub const fn destination_memory(&self) -> Option<Memory32> {
        self.operand.memory()
    }

    /// Gets the signed byte immediate.
    pub const fn immediate(&self) -> Immediate8 {
        self.immediate
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Sub
    }
}

impl SubRm32Imm8 {
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

impl Extent for SubRm32Imm8 {
    fn size(&self) -> Size {
        Size::new_valid(2 + self.operand.size().get())
    }
}

impl Encode for SubRm32Imm8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x83)?;
        self.operand.encode(encoder)?;
        self.immediate.encode(encoder)
    }
}

impl Display for SubRm32Imm8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::immediate::Immediate8;
    use crate::instruction::operand::register::Register32;
    use crate::instruction::operation::{Error, Operation};
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn exposes_destination_memory() {
        for (bytes, memory) in [
            (&b"\x83\x6c\x8b\xf0\xff"[..], "[ebx+ecx*4-16]"),
            (&b"\x83\x2d\x38\x81\x5d\x00\xff"[..], "[0x005d8138]"),
        ] {
            let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
            let Operation::SubRm32Imm8(operation) = operation else {
                panic!("expected SUB");
            };
            assert_eq!(operation.destination_register(), None);
            assert_eq!(operation.destination_memory().unwrap().to_string(), memory);
            assert_eq!(operation.immediate(), Immediate8::new(-1));
        }
    }

    #[test]
    fn rejects_truncation() {
        for bytes in [
            &b"\x83\xec\x58"[..],
            &b"\x83\x6c\x24\xf0\x58"[..],
            &b"\x83\xac\x8b\xf0\xff\xff\xff\x58"[..],
            &b"\x83\x2d\x01\x02\x03\x04\x58"[..],
        ] {
            for length in 1..bytes.len() {
                assert!(matches!(
                    Operation::decode_with(&mut &bytes[..length], &Prefixes::new()),
                    Err(Error::Decode(_)),
                ));
            }
        }
    }

    #[test]
    fn supports_register_and_memory_forms() {
        for (bytes, display) in [
            (&b"\x83\xec\x58"[..], "sub esp, 88"),
            (&b"\x83\xe8\xff"[..], "sub eax, -1"),
            (&b"\x83\x2b\x01"[..], "sub dword ptr [ebx], 1"),
            (&b"\x83\x6b\xf0\x80"[..], "sub dword ptr [ebx-16], -128"),
            (
                &b"\x83\xab\xf0\xff\xff\xff\x7f"[..],
                "sub dword ptr [ebx-16], 127",
            ),
            (
                &b"\x83\x2d\x78\x56\x34\x12\x00"[..],
                "sub dword ptr [0x12345678], 0",
            ),
            (&b"\x83\x2c\x24\x01"[..], "sub dword ptr [esp], 1"),
            (&b"\x83\x6d\x00\x01"[..], "sub dword ptr [ebp], 1"),
            (&b"\x83\x2c\x8b\x01"[..], "sub dword ptr [ebx+ecx*4], 1"),
            (
                &b"\x83\x6c\x8b\xf0\x01"[..],
                "sub dword ptr [ebx+ecx*4-16], 1",
            ),
            (
                &b"\x83\xac\x8b\xf0\xff\xff\xff\x01"[..],
                "sub dword ptr [ebx+ecx*4-16], 1",
            ),
            (
                &b"\x83\x2c\x8d\x10\x00\x00\x00\x01"[..],
                "sub dword ptr [ecx*4+16], 1",
            ),
            (
                &b"\x83\x2c\xe5\x78\x56\x34\x12\x01"[..],
                "sub dword ptr [0x12345678], 1",
            ),
        ] {
            let mut decoder = bytes;
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoded = Vec::new();

            operation.encode(&mut encoded).unwrap();

            assert_eq!(operation.mnemonic(), Mnemonic::Sub);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), bytes.len() as u64);
            assert_eq!(encoded, bytes);
            assert!(decoder.is_empty());

            let Operation::SubRm32Imm8(operation) = operation else {
                panic!("expected SUB");
            };

            assert_ne!(
                operation.destination_register().is_some(),
                operation.destination_memory().is_some(),
            );
            assert_eq!(operation.immediate().value(), *bytes.last().unwrap() as i8);
        }

        for register in 0..8 {
            for immediate in i8::MIN..=i8::MAX {
                let bytes = [0x83, 0xe8 | register, immediate as u8];
                let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                let Operation::SubRm32Imm8(operation) = operation else {
                    panic!("expected SUB");
                };

                assert_eq!(
                    operation.destination_register(),
                    Some(Register32::from_index(register))
                );
                assert_eq!(operation.immediate(), Immediate8::new(immediate));

                let mut encoded = Vec::new();

                Operation::SubRm32Imm8(operation)
                    .encode(&mut encoded)
                    .unwrap();

                assert_eq!(encoded, bytes);
            }
        }
    }
}
