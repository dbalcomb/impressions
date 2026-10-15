//! The x86 instruction operation.

pub mod encoding;
pub mod modifier;

mod error;

use std::fmt::{self, Display};

use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use super::Mnemonic;
use super::operand::immediate::Immediate8;
use super::operand::register::Register32;

use self::encoding::R32Rm32;

pub use self::error::Error;

/// An x86 instruction operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Push r32.
    ///
    /// `50+rd => push r32`
    PushR32(Register32),

    /// Push a 1-byte immediate, sign-extended to 32 bits.
    ///
    /// `6A ib => push imm8`
    PushImm8(Immediate8),

    /// Move r/m32 to r32.
    ///
    /// `8B /r => mov r32, r/m32`
    MovR32Rm32(R32Rm32),
}

impl Operation {
    /// Gets the mnemonic of the operation.
    pub const fn mnemonic(&self) -> Mnemonic {
        match self {
            Self::PushR32(_) => Mnemonic::Push,
            Self::PushImm8(_) => Mnemonic::Push,
            Self::MovR32Rm32(_) => Mnemonic::Mov,
        }
    }
}

impl Extent for Operation {
    fn size(&self) -> Size {
        match self {
            Self::PushR32(_) => Size::new_valid(1),
            Self::PushImm8(_) => Size::new_valid(2),
            Self::MovR32Rm32(operands) => Size::new_valid(1 + operands.size().get()),
        }
    }
}

impl Encode for Operation {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        match self {
            Self::PushR32(reg) => encoder.write_u8(0x50 | reg.index()),
            Self::PushImm8(immediate) => {
                encoder.write_u8(0x6a)?;
                immediate.encode(encoder)
            }
            Self::MovR32Rm32(operands) => {
                encoder.write_u8(0x8b)?;
                operands.encode(encoder)
            }
        }
    }
}

impl Decode for Operation {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        match decoder.read_u8()? {
            byte @ 0x50..0x58 => Ok(Self::PushR32(Register32::from_index(byte - 0x50))),
            0x6a => Ok(Self::PushImm8(Immediate8::decode(decoder)?)),
            0x8b => Ok(Self::MovR32Rm32(R32Rm32::decode(decoder)?)),
            opcode => Err(Error::InvalidOpcode(opcode)),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PushR32(reg) => write!(f, "{} {reg}", self.mnemonic()),
            Self::PushImm8(immediate) => write!(f, "{} {immediate}", self.mnemonic()),
            Self::MovR32Rm32(operands) => write!(f, "{} {operands}", self.mnemonic()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::operand::immediate::Immediate8;
    use crate::instruction::operand::register::Register32;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    use super::{Mnemonic, Operation};

    #[test]
    fn push_r32_supports_all_register_forms() {
        for (opcode, register, display) in [
            (0x50, Register32::Eax, "push eax"),
            (0x51, Register32::Ecx, "push ecx"),
            (0x52, Register32::Edx, "push edx"),
            (0x53, Register32::Ebx, "push ebx"),
            (0x54, Register32::Esp, "push esp"),
            (0x55, Register32::Ebp, "push ebp"),
            (0x56, Register32::Esi, "push esi"),
            (0x57, Register32::Edi, "push edi"),
        ] {
            let bytes = [opcode];
            let mut decoder = bytes.as_slice();
            let operation = Operation::decode(&mut decoder).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(operation, Operation::PushR32(register));
            assert_eq!(operation.mnemonic(), Mnemonic::Push);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), 1);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());
        }
    }

    #[test]
    fn push_imm8_supports_all_byte_values() {
        for value in i8::MIN..=i8::MAX {
            let bytes = [0x6a, value as u8];
            let mut decoder = bytes.as_slice();
            let operation = Operation::decode(&mut decoder).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(operation, Operation::PushImm8(Immediate8::new(value)));
            assert_eq!(operation.mnemonic(), Mnemonic::Push);
            assert_eq!(operation.to_string(), format!("push {value}"));
            assert_eq!(operation.size(), 2);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());

            let Operation::PushImm8(immediate) = operation else {
                panic!("expected immediate push");
            };

            assert_eq!(immediate.value(), value);
        }
    }

    #[test]
    fn mov_r32_rm32_supports_all_register_and_memory_forms() {
        for (bytes, display) in [
            (&b"\x8b\xc1"[..], "mov eax, ecx"),
            (&b"\x8b\x44\x8b\xf0"[..], "mov eax, [ebx+ecx*4-16]"),
            (&b"\x8b\x03"[..], "mov eax, [ebx]"),
            (&b"\x8b\x43\x10"[..], "mov eax, [ebx+16]"),
            (&b"\x8b\x83\xf0\xff\xff\xff"[..], "mov eax, [ebx-16]"),
            (&b"\x8b\x05\x78\x56\x34\x12"[..], "mov eax, [0x12345678]"),
            (&b"\x8b\x04\x24"[..], "mov eax, [esp]"),
            (&b"\x8b\x45\x00"[..], "mov eax, [ebp]"),
            (&b"\x8b\x04\x8b"[..], "mov eax, [ebx+ecx*4]"),
            (
                &b"\x8b\x84\x8b\xf0\xff\xff\xff"[..],
                "mov eax, [ebx+ecx*4-16]",
            ),
            (&b"\x8b\x44\x24\xf0"[..], "mov eax, [esp-16]"),
            (&b"\x8b\x84\x24\xf0\xff\xff\xff"[..], "mov eax, [esp-16]"),
            (&b"\x8b\x04\x8d\x10\x00\x00\x00"[..], "mov eax, [ecx*4+16]"),
            (
                &b"\x8b\x04\x25\x78\x56\x34\x12"[..],
                "mov eax, [0x12345678]",
            ),
            (&b"\x8b\xd3"[..], "mov edx, ebx"),
            (&b"\x8b\x43\x00"[..], "mov eax, [ebx]"),
            (&b"\x8b\x83\x00\x00\x00\x00"[..], "mov eax, [ebx]"),
            (&b"\x8b\x04\x0b"[..], "mov eax, [ebx+ecx*1]"),
            (&b"\x8b\x04\x4b"[..], "mov eax, [ebx+ecx*2]"),
            (&b"\x8b\x04\xcb"[..], "mov eax, [ebx+ecx*8]"),
            (&b"\x8b\x04\xe3"[..], "mov eax, [ebx]"),
            (&b"\x8b\x44\x25\x00"[..], "mov eax, [ebp]"),
            (&b"\x8b\x84\x25\x00\x00\x00\x00"[..], "mov eax, [ebp]"),
            (&b"\x8b\x85\x00\x00\x00\x00"[..], "mov eax, [ebp]"),
        ] {
            let mut decoder = bytes;
            let operation = Operation::decode(&mut decoder).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(operation.mnemonic(), Mnemonic::Mov);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), bytes.len() as u64);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());
        }
    }
}
