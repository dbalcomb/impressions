use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::immediate::Immediate8;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use crate::instruction::operation::encoding::Rm32;

/// Bitwise AND r/m32 with a sign-extended byte immediate.
///
/// `83 /4 ib => and r/m32, imm8`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AndRm32Imm8 {
    operand: Rm32,
    immediate: Immediate8,
}

impl AndRm32Imm8 {
    /// Creates the operation from its encoded operands.
    pub const fn new(operand: Rm32, immediate: Immediate8) -> Self {
        Self { operand, immediate }
    }

    /// Gets the register or memory operand.
    pub const fn operand(&self) -> Rm32 {
        self.operand
    }

    /// Gets the signed byte immediate.
    pub const fn immediate(&self) -> Immediate8 {
        self.immediate
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::And
    }
}

impl AndRm32Imm8 {
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

impl Extent for AndRm32Imm8 {
    fn size(&self) -> Size {
        Size::new_valid(2 + self.operand.size().get())
    }
}

impl Encode for AndRm32Imm8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x83)?;
        self.operand.encode(encoder)?;
        self.immediate.encode(encoder)
    }
}

impl Display for AndRm32Imm8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::immediate::Immediate8;
    use crate::instruction::operand::register::Register32;
    use crate::instruction::operation::Operation;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn supports_register_and_memory_operands() {
        for (operand_bytes, destination) in [
            (&b"\xc0"[..], "eax"),
            (&b"\x03"[..], "dword ptr [ebx]"),
            (&b"\x43\xf0"[..], "dword ptr [ebx-16]"),
            (&b"\x83\xf0\xff\xff\xff"[..], "dword ptr [ebx-16]"),
            (&b"\x04\x24"[..], "dword ptr [esp]"),
            (&b"\x45\x00"[..], "dword ptr [ebp]"),
            (&b"\x04\x8b"[..], "dword ptr [ebx+ecx*4]"),
            (&b"\x44\x8b\xf0"[..], "dword ptr [ebx+ecx*4-16]"),
            (&b"\x84\x8b\xf0\xff\xff\xff"[..], "dword ptr [ebx+ecx*4-16]"),
            (&b"\x04\x8d\x10\x00\x00\x00"[..], "dword ptr [ecx*4+16]"),
            (&b"\x04\xe5\x78\x56\x34\x12"[..], "dword ptr [0x12345678]"),
            (&b"\x05\x78\x56\x34\x12"[..], "dword ptr [0x12345678]"),
        ] {
            for value in [i8::MIN, -1, 0, 1, i8::MAX] {
                let mut bytes = vec![0x83];

                bytes.extend_from_slice(operand_bytes);
                bytes[1] |= 4 << 3;
                bytes.push(value as u8);

                let operation =
                    Operation::decode_with(&mut bytes.as_slice(), &Prefixes::new()).unwrap();
                let Operation::AndRm32Imm8(operation) = operation else {
                    panic!("expected AND");
                };

                assert_eq!(operation.immediate(), Immediate8::new(value));
                assert_eq!(operation.operand().opcode_extension(), 4);
                assert_eq!(operation.mnemonic(), Mnemonic::And);
                assert_eq!(operation.to_string(), format!("and {destination}, {value}"));
                assert_eq!(operation.size(), bytes.len() as u64);

                let mut encoded = Vec::new();

                operation.encode(&mut encoded).unwrap();

                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn supports_all_registers_and_byte_immediates() {
        for register in 0..8 {
            for value in i8::MIN..=i8::MAX {
                let bytes = [0x83, 0xe0 | register, value as u8];
                let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                let Operation::AndRm32Imm8(operation) = operation else {
                    panic!("expected AND");
                };

                assert_eq!(
                    operation.operand().rm_register(),
                    Some(Register32::from_index(register))
                );
                assert_eq!(operation.immediate(), Immediate8::new(value));

                let mut encoded = Vec::new();

                operation.encode(&mut encoded).unwrap();

                assert_eq!(encoded, bytes);
            }
        }
    }
}
