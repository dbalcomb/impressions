//! The x86 instruction operation.

pub mod encoding;
pub mod modifier;

mod error;

use std::fmt::{self, Display};

use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use super::Mnemonic;
use super::operand::immediate::{Immediate8, Immediate32};
use super::operand::register::Register32;
use super::prefix::Prefixes;

use self::encoding::{R32Rm32, Rm32, Rm32R32};

pub use self::error::Error;

/// An x86 instruction operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Push r32.
    ///
    /// `50+rd => push r32`
    PushR32(Register32),

    /// Push a 32-bit immediate.
    ///
    /// `68 id => push imm32`
    PushImm32(Immediate32),

    /// Push an 8-bit immediate, sign-extended to 32 bits.
    ///
    /// `6A ib => push imm8`
    PushImm8(Immediate8),

    /// Add a sign-extended byte immediate to r/m32.
    ///
    /// `83 /0 ib => add r/m32, imm8`
    AddRm32Imm8(Rm32, Immediate8),

    /// Bitwise OR r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /1 ib => or r/m32, imm8`
    OrRm32Imm8(Rm32, Immediate8),

    /// Add a sign-extended byte immediate and carry to r/m32.
    ///
    /// `83 /2 ib => adc r/m32, imm8`
    AdcRm32Imm8(Rm32, Immediate8),

    /// Subtract a sign-extended byte immediate and borrow from r/m32.
    ///
    /// `83 /3 ib => sbb r/m32, imm8`
    SbbRm32Imm8(Rm32, Immediate8),

    /// Bitwise AND r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /4 ib => and r/m32, imm8`
    AndRm32Imm8(Rm32, Immediate8),

    /// Subtract a sign-extended byte immediate from r/m32.
    ///
    /// `83 /5 ib => sub r/m32, imm8`
    SubRm32Imm8(Rm32, Immediate8),

    /// Bitwise XOR r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /6 ib => xor r/m32, imm8`
    XorRm32Imm8(Rm32, Immediate8),

    /// Compare r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /7 ib => cmp r/m32, imm8`
    CmpRm32Imm8(Rm32, Immediate8),

    /// Move r32 to r/m32.
    ///
    /// `89 /r => mov r/m32, r32`
    MovRm32R32(Rm32R32),

    /// Move r/m32 to r32.
    ///
    /// `8B /r => mov r32, r/m32`
    MovR32Rm32(R32Rm32),

    /// Move a dword at a direct 32-bit memory offset to EAX.
    ///
    /// `A1 moffs32 => mov eax, moffs32`
    MovEaxMoffs32(Address),
}

impl Operation {
    /// Gets the mnemonic of the operation.
    pub const fn mnemonic(&self) -> Mnemonic {
        match self {
            Self::PushR32(_) => Mnemonic::Push,
            Self::PushImm32(_) => Mnemonic::Push,
            Self::PushImm8(_) => Mnemonic::Push,
            Self::AddRm32Imm8(..) => Mnemonic::Add,
            Self::OrRm32Imm8(..) => Mnemonic::Or,
            Self::AdcRm32Imm8(..) => Mnemonic::Adc,
            Self::SbbRm32Imm8(..) => Mnemonic::Sbb,
            Self::AndRm32Imm8(..) => Mnemonic::And,
            Self::SubRm32Imm8(..) => Mnemonic::Sub,
            Self::XorRm32Imm8(..) => Mnemonic::Xor,
            Self::CmpRm32Imm8(..) => Mnemonic::Cmp,
            Self::MovRm32R32(_) => Mnemonic::Mov,
            Self::MovR32Rm32(_) => Mnemonic::Mov,
            Self::MovEaxMoffs32(_) => Mnemonic::Mov,
        }
    }
}

impl Extent for Operation {
    fn size(&self) -> Size {
        match self {
            Self::PushR32(_) => Size::new_valid(1),
            Self::PushImm32(_) => Size::new_valid(5),
            Self::PushImm8(_) => Size::new_valid(2),
            Self::AddRm32Imm8(operand, _)
            | Self::OrRm32Imm8(operand, _)
            | Self::AdcRm32Imm8(operand, _)
            | Self::SbbRm32Imm8(operand, _)
            | Self::AndRm32Imm8(operand, _)
            | Self::SubRm32Imm8(operand, _)
            | Self::XorRm32Imm8(operand, _)
            | Self::CmpRm32Imm8(operand, _) => Size::new_valid(2 + operand.size().get()),
            Self::MovRm32R32(operands) => Size::new_valid(1 + operands.size().get()),
            Self::MovR32Rm32(operands) => Size::new_valid(1 + operands.size().get()),
            Self::MovEaxMoffs32(_) => Size::new_valid(5),
        }
    }
}

impl Encode for Operation {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        match self {
            Self::PushR32(reg) => encoder.write_u8(0x50 | reg.index()),
            Self::PushImm32(immediate) => {
                encoder.write_u8(0x68)?;
                immediate.encode(encoder)
            }
            Self::PushImm8(immediate) => {
                encoder.write_u8(0x6a)?;
                immediate.encode(encoder)
            }
            Self::AddRm32Imm8(operand, immediate)
            | Self::OrRm32Imm8(operand, immediate)
            | Self::AdcRm32Imm8(operand, immediate)
            | Self::SbbRm32Imm8(operand, immediate)
            | Self::AndRm32Imm8(operand, immediate)
            | Self::SubRm32Imm8(operand, immediate)
            | Self::XorRm32Imm8(operand, immediate)
            | Self::CmpRm32Imm8(operand, immediate) => {
                encoder.write_u8(0x83)?;
                operand.encode(encoder)?;
                immediate.encode(encoder)
            }
            Self::MovRm32R32(operands) => {
                encoder.write_u8(0x89)?;
                operands.encode(encoder)
            }
            Self::MovR32Rm32(operands) => {
                encoder.write_u8(0x8b)?;
                operands.encode(encoder)
            }
            Self::MovEaxMoffs32(offset) => {
                encoder.write_u8(0xa1)?;
                offset.encode(encoder)
            }
        }
    }
}

impl Decode for Operation {
    type Context<'a> = &'a Prefixes;
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        match decoder.read_u8()? {
            byte @ 0x50..0x58 => Ok(Self::PushR32(Register32::from_index(byte - 0x50))),
            0x68 => Ok(Self::PushImm32(Immediate32::decode(decoder)?)),
            0x6a => Ok(Self::PushImm8(Immediate8::decode(decoder)?)),
            0x83 => {
                let operand = Rm32::decode(decoder)?;
                let immediate = Immediate8::decode(decoder)?;

                match operand.opcode_extension() {
                    0 => Ok(Self::AddRm32Imm8(operand, immediate)),
                    1 => Ok(Self::OrRm32Imm8(operand, immediate)),
                    2 => Ok(Self::AdcRm32Imm8(operand, immediate)),
                    3 => Ok(Self::SbbRm32Imm8(operand, immediate)),
                    4 => Ok(Self::AndRm32Imm8(operand, immediate)),
                    5 => Ok(Self::SubRm32Imm8(operand, immediate)),
                    6 => Ok(Self::XorRm32Imm8(operand, immediate)),
                    7 => Ok(Self::CmpRm32Imm8(operand, immediate)),
                    _ => unreachable!("ModR/M opcode extension is three bits"),
                }
            }
            0x89 => Ok(Self::MovRm32R32(Rm32R32::decode(decoder)?)),
            0x8b => Ok(Self::MovR32Rm32(R32Rm32::decode(decoder)?)),
            0xa1 => Ok(Self::MovEaxMoffs32(Address::decode(decoder)?)),
            opcode => Err(Error::InvalidOpcode(opcode)),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PushR32(reg) => write!(f, "{} {reg}", self.mnemonic()),
            Self::PushImm32(immediate) => write!(f, "{} {immediate}", self.mnemonic()),
            Self::PushImm8(immediate) => write!(f, "{} {immediate}", self.mnemonic()),
            Self::AddRm32Imm8(operand, immediate)
            | Self::OrRm32Imm8(operand, immediate)
            | Self::AdcRm32Imm8(operand, immediate)
            | Self::SbbRm32Imm8(operand, immediate)
            | Self::AndRm32Imm8(operand, immediate)
            | Self::SubRm32Imm8(operand, immediate)
            | Self::XorRm32Imm8(operand, immediate)
            | Self::CmpRm32Imm8(operand, immediate) => {
                write!(f, "{} ", self.mnemonic())?;

                if operand.memory().is_some() {
                    f.write_str("dword ptr ")?;
                }

                write!(f, "{operand}, {immediate}")
            }
            Self::MovRm32R32(operands) => write!(f, "{} {operands}", self.mnemonic()),
            Self::MovR32Rm32(operands) => write!(f, "{} {operands}", self.mnemonic()),
            Self::MovEaxMoffs32(offset) => write!(f, "{} eax, [{offset}]", self.mnemonic()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::operand::immediate::{Immediate8, Immediate32};
    use crate::instruction::operand::register::Register32;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::address::Address;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    use super::{Mnemonic, Operation};

    #[test]
    fn sub_rm32_imm8_supports_register_and_memory_forms() {
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

            let Operation::SubRm32Imm8(operands, immediate) = operation else {
                panic!("expected SUB");
            };

            assert_eq!(operands.opcode_extension(), 5);
            assert_eq!(immediate.value(), *bytes.last().unwrap() as i8);
        }

        for register in 0..8 {
            for immediate in i8::MIN..=i8::MAX {
                let bytes = [0x83, 0xe8 | register, immediate as u8];
                let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                let Operation::SubRm32Imm8(operands, decoded_immediate) = operation else {
                    panic!("expected SUB");
                };

                assert_eq!(
                    operands.rm_register(),
                    Some(Register32::from_index(register))
                );
                assert_eq!(decoded_immediate, Immediate8::new(immediate));

                let mut encoded = Vec::new();

                Operation::SubRm32Imm8(operands, decoded_immediate)
                    .encode(&mut encoded)
                    .unwrap();

                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn group83_rejects_truncation() {
        for bytes in [
            &b"\x83"[..],
            &b"\x83\xec"[..],
            &b"\x83\x2c"[..],
            &b"\x83\x6c\x24"[..],
            &b"\x83\x6c\x24\xf0"[..],
            &b"\x83\x2d\x01\x02\x03"[..],
            &b"\x83\x2d\x01\x02\x03\x04"[..],
        ] {
            assert!(matches!(
                Operation::decode_with(&mut &bytes[..], &Prefixes::new()),
                Err(super::Error::Decode(_)),
            ));
        }
    }

    #[test]
    fn group83_supports_all_operations_and_addressing_forms() {
        for (extension, mnemonic, name, constructor) in [
            (
                0,
                Mnemonic::Add,
                "add",
                Operation::AddRm32Imm8 as fn(super::Rm32, Immediate8) -> Operation,
            ),
            (1, Mnemonic::Or, "or", Operation::OrRm32Imm8),
            (2, Mnemonic::Adc, "adc", Operation::AdcRm32Imm8),
            (3, Mnemonic::Sbb, "sbb", Operation::SbbRm32Imm8),
            (4, Mnemonic::And, "and", Operation::AndRm32Imm8),
            (5, Mnemonic::Sub, "sub", Operation::SubRm32Imm8),
            (6, Mnemonic::Xor, "xor", Operation::XorRm32Imm8),
            (7, Mnemonic::Cmp, "cmp", Operation::CmpRm32Imm8),
        ] {
            for (operand_bytes, destination) in [
                (&b"\xc0"[..], "eax"),
                (&b"\x03"[..], "dword ptr [ebx]"),
                (&b"\x43\xf0"[..], "dword ptr [ebx-16]"),
                (&b"\x83\xf0\xff\xff\xff"[..], "dword ptr [ebx-16]"),
                (&b"\x05\x78\x56\x34\x12"[..], "dword ptr [0x12345678]"),
                (&b"\x04\x24"[..], "dword ptr [esp]"),
                (&b"\x45\x00"[..], "dword ptr [ebp]"),
                (&b"\x04\x8b"[..], "dword ptr [ebx+ecx*4]"),
                (&b"\x44\x8b\xf0"[..], "dword ptr [ebx+ecx*4-16]"),
                (&b"\x84\x8b\xf0\xff\xff\xff"[..], "dword ptr [ebx+ecx*4-16]"),
                (&b"\x04\x8d\x10\x00\x00\x00"[..], "dword ptr [ecx*4+16]"),
                (&b"\x04\xe5\x78\x56\x34\x12"[..], "dword ptr [0x12345678]"),
            ] {
                for immediate in [i8::MIN, -1, 0, 1, i8::MAX] {
                    let mut bytes = vec![0x83];

                    bytes.extend_from_slice(operand_bytes);
                    bytes[1] |= extension << 3;
                    bytes.push(immediate as u8);

                    let mut decoder = bytes.as_slice();
                    let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
                    let operands = super::Rm32::decode(&mut &bytes[1..]).unwrap();

                    assert_eq!(operation, constructor(operands, Immediate8::new(immediate)));
                    assert_eq!(operation.mnemonic(), mnemonic);
                    assert_eq!(
                        operation.to_string(),
                        format!("{name} {destination}, {immediate}")
                    );
                    assert_eq!(operation.size(), bytes.len() as u64);
                    assert_eq!(operands.opcode_extension(), extension);

                    let mut encoded = Vec::new();

                    operation.encode(&mut encoded).unwrap();

                    assert_eq!(encoded, bytes);
                    assert!(decoder.is_empty());
                }
            }

            for register in 0..8 {
                for immediate in i8::MIN..=i8::MAX {
                    let bytes = [0x83, 0xc0 | extension << 3 | register, immediate as u8];
                    let operation =
                        Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                    let operands = super::Rm32::decode(&mut &bytes[1..]).unwrap();

                    assert_eq!(operation, constructor(operands, Immediate8::new(immediate)));
                    assert_eq!(
                        operands.rm_register(),
                        Some(Register32::from_index(register))
                    );

                    let mut encoded = Vec::new();

                    operation.encode(&mut encoded).unwrap();

                    assert_eq!(encoded, bytes);
                }
            }
        }
    }

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
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
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
    fn push_imm32_supports_signed_values_and_little_endian_encoding() {
        for (bytes, value, display) in [
            (&b"\x68\x00\x00\x00\x00"[..], 0, "push 0"),
            (&b"\x68\x01\x00\x00\x00"[..], 1, "push 1"),
            (&b"\x68\xff\xff\xff\xff"[..], -1, "push -1"),
            (&b"\x68\x80\xff\xff\xff"[..], -128, "push -128"),
            (&b"\x68\x7f\x00\x00\x00"[..], 127, "push 127"),
            (&b"\x68\x78\x56\x34\x12"[..], 0x12345678, "push 305419896"),
            (&b"\x68\x00\x00\x00\x80"[..], i32::MIN, "push -2147483648"),
            (&b"\x68\xff\xff\xff\x7f"[..], i32::MAX, "push 2147483647"),
        ] {
            let mut decoder = bytes;
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(operation, Operation::PushImm32(Immediate32::new(value)));
            assert_eq!(operation.mnemonic(), Mnemonic::Push);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), 5);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());

            let Operation::PushImm32(immediate) = operation else {
                panic!("expected immediate push");
            };

            assert_eq!(immediate.value(), value);
        }
    }

    #[test]
    fn push_imm8_supports_all_byte_values() {
        for value in i8::MIN..=i8::MAX {
            let bytes = [0x6a, value as u8];
            let mut decoder = bytes.as_slice();
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
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
    fn mov_rm32_r32_supports_all_register_and_memory_forms() {
        for (bytes, display) in [
            (&b"\x89\xc1"[..], "mov ecx, eax"),
            (&b"\x89\xd3"[..], "mov ebx, edx"),
            (&b"\x89\x03"[..], "mov [ebx], eax"),
            (&b"\x89\x43\x10"[..], "mov [ebx+16], eax"),
            (&b"\x89\x83\xf0\xff\xff\xff"[..], "mov [ebx-16], eax"),
            (&b"\x89\x25\x00\x00\x00\x00"[..], "mov [0x00000000], esp"),
            (&b"\x89\x04\x24"[..], "mov [esp], eax"),
            (&b"\x89\x45\x00"[..], "mov [ebp], eax"),
            (&b"\x89\x85\x00\x00\x00\x00"[..], "mov [ebp], eax"),
            (&b"\x89\x04\x8b"[..], "mov [ebx+ecx*4], eax"),
            (&b"\x89\x44\x8b\xf0"[..], "mov [ebx+ecx*4-16], eax"),
            (
                &b"\x89\x84\x8b\xf0\xff\xff\xff"[..],
                "mov [ebx+ecx*4-16], eax",
            ),
            (&b"\x89\x44\x24\xf0"[..], "mov [esp-16], eax"),
            (&b"\x89\x84\x24\xf0\xff\xff\xff"[..], "mov [esp-16], eax"),
            (&b"\x89\x04\x8d\x10\x00\x00\x00"[..], "mov [ecx*4+16], eax"),
            (
                &b"\x89\x04\x25\x78\x56\x34\x12"[..],
                "mov [0x12345678], eax",
            ),
            (&b"\x89\x43\x00"[..], "mov [ebx], eax"),
            (&b"\x89\x83\x00\x00\x00\x00"[..], "mov [ebx], eax"),
            (&b"\x89\x04\x0b"[..], "mov [ebx+ecx*1], eax"),
            (&b"\x89\x04\x4b"[..], "mov [ebx+ecx*2], eax"),
            (&b"\x89\x04\xcb"[..], "mov [ebx+ecx*8], eax"),
            (&b"\x89\x04\xe3"[..], "mov [ebx], eax"),
            (&b"\x89\x44\x25\x00"[..], "mov [ebp], eax"),
            (&b"\x89\x84\x25\x00\x00\x00\x00"[..], "mov [ebp], eax"),
        ] {
            let mut decoder = bytes;
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert!(matches!(operation, Operation::MovRm32R32(_)));
            assert_eq!(operation.mnemonic(), Mnemonic::Mov);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), bytes.len() as u64);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());
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
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(operation.mnemonic(), Mnemonic::Mov);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), bytes.len() as u64);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());
        }
    }

    #[test]
    fn mov_eax_moffs32_supports_direct_offsets() {
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

            assert_eq!(operation, Operation::MovEaxMoffs32(Address::new(offset)));
            assert_eq!(operation.mnemonic(), Mnemonic::Mov);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), 5);
            assert_eq!(encoded, bytes);
            assert!(decoder.is_empty());
        }
    }
}
