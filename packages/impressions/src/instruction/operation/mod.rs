//! The x86 instruction operation.

pub mod types;

mod encoding;
mod error;
mod modifier;

use std::fmt::{self, Display};

use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use super::Mnemonic;
use super::operand::immediate::{Immediate8, Immediate32};
use super::operand::register::Register32;
use super::prefix::Prefixes;

use self::encoding::Rm32;
use self::types::adc::{AdcRm32Imm8, AdcRm32Imm32};
use self::types::add::{AddRm32Imm8, AddRm32Imm32};
use self::types::and::{AndRm32Imm8, AndRm32Imm32};
use self::types::call::CallRm32;
use self::types::cmp::{CmpRm32Imm8, CmpRm32Imm32};
use self::types::mov::{MovEaxMoffs32, MovR8Rm8, MovR32Rm32, MovRm32R32};
use self::types::or::{OrRm32Imm8, OrRm32Imm32};
use self::types::push::{PushImm8, PushImm32, PushR32};
use self::types::rcl::RclRm32Imm8;
use self::types::rcr::RcrRm32Imm8;
use self::types::rol::RolRm32Imm8;
use self::types::ror::RorRm32Imm8;
use self::types::sar::SarRm32Imm8;
use self::types::sbb::{SbbRm32Imm8, SbbRm32Imm32};
use self::types::shl::ShlRm32Imm8;
use self::types::shr::ShrRm32Imm8;
use self::types::sub::{SubRm32Imm8, SubRm32Imm32};
use self::types::xor::{XorR32Rm32, XorRm32Imm8, XorRm32Imm32};

pub use self::error::Error;

/// An x86 instruction operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Bitwise XOR a 32-bit register with a 32-bit register or memory value.
    ///
    /// `33 /r => xor r32, r/m32`
    XorR32Rm32(XorR32Rm32),

    /// Push a 32-bit register onto the stack.
    ///
    /// `50+rd => push r32`
    PushR32(PushR32),

    /// Push a 32-bit immediate onto the stack.
    ///
    /// `68 id => push imm32`
    PushImm32(PushImm32),

    /// Push an 8-bit immediate onto the stack, sign-extended to 32 bits.
    ///
    /// `6A ib => push imm8`
    PushImm8(PushImm8),

    /// Add a 32-bit immediate to r/m32.
    ///
    /// `81 /0 id => add r/m32, imm32`
    AddRm32Imm32(AddRm32Imm32),

    /// Bitwise OR r/m32 with a 32-bit immediate.
    ///
    /// `81 /1 id => or r/m32, imm32`
    OrRm32Imm32(OrRm32Imm32),

    /// Add a 32-bit immediate and carry to r/m32.
    ///
    /// `81 /2 id => adc r/m32, imm32`
    AdcRm32Imm32(AdcRm32Imm32),

    /// Subtract a 32-bit immediate and borrow from r/m32.
    ///
    /// `81 /3 id => sbb r/m32, imm32`
    SbbRm32Imm32(SbbRm32Imm32),

    /// Bitwise AND r/m32 with a 32-bit immediate.
    ///
    /// `81 /4 id => and r/m32, imm32`
    AndRm32Imm32(AndRm32Imm32),

    /// Subtract a 32-bit immediate from r/m32.
    ///
    /// `81 /5 id => sub r/m32, imm32`
    SubRm32Imm32(SubRm32Imm32),

    /// Bitwise XOR r/m32 with a 32-bit immediate.
    ///
    /// `81 /6 id => xor r/m32, imm32`
    XorRm32Imm32(XorRm32Imm32),

    /// Compare r/m32 with a 32-bit immediate.
    ///
    /// `81 /7 id => cmp r/m32, imm32`
    CmpRm32Imm32(CmpRm32Imm32),

    /// Add a sign-extended byte immediate to r/m32.
    ///
    /// `83 /0 ib => add r/m32, imm8`
    AddRm32Imm8(AddRm32Imm8),

    /// Bitwise OR r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /1 ib => or r/m32, imm8`
    OrRm32Imm8(OrRm32Imm8),

    /// Add a sign-extended byte immediate and carry to r/m32.
    ///
    /// `83 /2 ib => adc r/m32, imm8`
    AdcRm32Imm8(AdcRm32Imm8),

    /// Subtract a sign-extended byte immediate and borrow from r/m32.
    ///
    /// `83 /3 ib => sbb r/m32, imm8`
    SbbRm32Imm8(SbbRm32Imm8),

    /// Bitwise AND r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /4 ib => and r/m32, imm8`
    AndRm32Imm8(AndRm32Imm8),

    /// Subtract a sign-extended byte immediate from r/m32.
    ///
    /// `83 /5 ib => sub r/m32, imm8`
    SubRm32Imm8(SubRm32Imm8),

    /// Bitwise XOR r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /6 ib => xor r/m32, imm8`
    XorRm32Imm8(XorRm32Imm8),

    /// Compare r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /7 ib => cmp r/m32, imm8`
    CmpRm32Imm8(CmpRm32Imm8),

    /// Move a 32-bit register value into a 32-bit register or memory location.
    ///
    /// `89 /r => mov r/m32, r32`
    MovRm32R32(MovRm32R32),

    /// Move an 8-bit register or memory value into an 8-bit register.
    ///
    /// `8A /r => mov r8, r/m8`
    MovR8Rm8(MovR8Rm8),

    /// Move a 32-bit register or memory value into a 32-bit register.
    ///
    /// `8B /r => mov r32, r/m32`
    MovR32Rm32(MovR32Rm32),

    /// Move a dword at a direct 32-bit memory offset to EAX.
    ///
    /// `A1 moffs32 => mov eax, moffs32`
    MovEaxMoffs32(MovEaxMoffs32),

    /// Rotate a 32-bit register or memory value left by an unsigned byte count.
    ///
    /// `C1 /0 ib => rol r/m32, imm8`
    RolRm32Imm8(RolRm32Imm8),

    /// Rotate a 32-bit register or memory value right by an unsigned byte
    /// count.
    ///
    /// `C1 /1 ib => ror r/m32, imm8`
    RorRm32Imm8(RorRm32Imm8),

    /// Rotate a 32-bit register or memory value left through carry by an
    /// unsigned byte count.
    ///
    /// `C1 /2 ib => rcl r/m32, imm8`
    RclRm32Imm8(RclRm32Imm8),

    /// Rotate a 32-bit register or memory value right through carry by an
    /// unsigned byte count.
    ///
    /// `C1 /3 ib => rcr r/m32, imm8`
    RcrRm32Imm8(RcrRm32Imm8),

    /// Shift a 32-bit register or memory value left by an unsigned byte count.
    ///
    /// `C1 /4 ib => shl r/m32, imm8`
    ShlRm32Imm8(ShlRm32Imm8),

    /// Shift a 32-bit register or memory value right logically by an unsigned
    /// byte count.
    ///
    /// `C1 /5 ib => shr r/m32, imm8`
    ShrRm32Imm8(ShrRm32Imm8),

    /// Shift a 32-bit register or memory value right arithmetically by an
    /// unsigned byte count.
    ///
    /// `C1 /7 ib => sar r/m32, imm8`
    SarRm32Imm8(SarRm32Imm8),

    /// Call a near procedure indirectly through r/m32.
    ///
    /// `FF /2 => call r/m32`
    CallRm32(CallRm32),
}

impl Operation {
    /// Gets the mnemonic of the operation.
    pub const fn mnemonic(&self) -> Mnemonic {
        match self {
            Self::XorR32Rm32(operation) => operation.mnemonic(),
            Self::PushR32(operation) => operation.mnemonic(),
            Self::PushImm32(operation) => operation.mnemonic(),
            Self::PushImm8(operation) => operation.mnemonic(),
            Self::AddRm32Imm32(operation) => operation.mnemonic(),
            Self::OrRm32Imm32(operation) => operation.mnemonic(),
            Self::AdcRm32Imm32(operation) => operation.mnemonic(),
            Self::SbbRm32Imm32(operation) => operation.mnemonic(),
            Self::AndRm32Imm32(operation) => operation.mnemonic(),
            Self::SubRm32Imm32(operation) => operation.mnemonic(),
            Self::XorRm32Imm32(operation) => operation.mnemonic(),
            Self::CmpRm32Imm32(operation) => operation.mnemonic(),
            Self::AddRm32Imm8(operation) => operation.mnemonic(),
            Self::OrRm32Imm8(operation) => operation.mnemonic(),
            Self::AdcRm32Imm8(operation) => operation.mnemonic(),
            Self::SbbRm32Imm8(operation) => operation.mnemonic(),
            Self::AndRm32Imm8(operation) => operation.mnemonic(),
            Self::SubRm32Imm8(operation) => operation.mnemonic(),
            Self::XorRm32Imm8(operation) => operation.mnemonic(),
            Self::CmpRm32Imm8(operation) => operation.mnemonic(),
            Self::MovRm32R32(operation) => operation.mnemonic(),
            Self::MovR8Rm8(operation) => operation.mnemonic(),
            Self::MovR32Rm32(operation) => operation.mnemonic(),
            Self::MovEaxMoffs32(operation) => operation.mnemonic(),
            Self::RolRm32Imm8(operation) => operation.mnemonic(),
            Self::RorRm32Imm8(operation) => operation.mnemonic(),
            Self::RclRm32Imm8(operation) => operation.mnemonic(),
            Self::RcrRm32Imm8(operation) => operation.mnemonic(),
            Self::ShlRm32Imm8(operation) => operation.mnemonic(),
            Self::ShrRm32Imm8(operation) => operation.mnemonic(),
            Self::SarRm32Imm8(operation) => operation.mnemonic(),
            Self::CallRm32(operation) => operation.mnemonic(),
        }
    }
}

impl Operation {
    pub(super) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        match self {
            Self::XorR32Rm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::PushR32(operation) => Display::fmt(operation, f),
            Self::PushImm32(operation) => Display::fmt(operation, f),
            Self::PushImm8(operation) => Display::fmt(operation, f),
            Self::AddRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::OrRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::AdcRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::SbbRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::AndRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::SubRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::XorRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::CmpRm32Imm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::AddRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::OrRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::AdcRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::SbbRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::AndRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::SubRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::XorRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::CmpRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::MovRm32R32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::MovR8Rm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::MovR32Rm32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::MovEaxMoffs32(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::RolRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::RorRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::RclRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::RcrRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::ShlRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::ShrRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::SarRm32Imm8(operation) => operation.fmt_with_prefixes(f, prefixes),
            Self::CallRm32(operation) => operation.fmt_with_prefixes(f, prefixes),
        }
    }
}

impl Extent for Operation {
    fn size(&self) -> Size {
        match self {
            Self::XorR32Rm32(operation) => operation.size(),
            Self::PushR32(operation) => operation.size(),
            Self::PushImm32(operation) => operation.size(),
            Self::PushImm8(operation) => operation.size(),
            Self::AddRm32Imm32(operation) => operation.size(),
            Self::OrRm32Imm32(operation) => operation.size(),
            Self::AdcRm32Imm32(operation) => operation.size(),
            Self::SbbRm32Imm32(operation) => operation.size(),
            Self::AndRm32Imm32(operation) => operation.size(),
            Self::SubRm32Imm32(operation) => operation.size(),
            Self::XorRm32Imm32(operation) => operation.size(),
            Self::CmpRm32Imm32(operation) => operation.size(),
            Self::AddRm32Imm8(operation) => operation.size(),
            Self::OrRm32Imm8(operation) => operation.size(),
            Self::AdcRm32Imm8(operation) => operation.size(),
            Self::SbbRm32Imm8(operation) => operation.size(),
            Self::AndRm32Imm8(operation) => operation.size(),
            Self::SubRm32Imm8(operation) => operation.size(),
            Self::XorRm32Imm8(operation) => operation.size(),
            Self::CmpRm32Imm8(operation) => operation.size(),
            Self::MovRm32R32(operation) => operation.size(),
            Self::MovR8Rm8(operation) => operation.size(),
            Self::MovR32Rm32(operation) => operation.size(),
            Self::MovEaxMoffs32(operation) => operation.size(),
            Self::RolRm32Imm8(operation) => operation.size(),
            Self::RorRm32Imm8(operation) => operation.size(),
            Self::RclRm32Imm8(operation) => operation.size(),
            Self::RcrRm32Imm8(operation) => operation.size(),
            Self::ShlRm32Imm8(operation) => operation.size(),
            Self::ShrRm32Imm8(operation) => operation.size(),
            Self::SarRm32Imm8(operation) => operation.size(),
            Self::CallRm32(operation) => operation.size(),
        }
    }
}

impl Encode for Operation {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        match self {
            Self::XorR32Rm32(operation) => operation.encode(encoder),
            Self::PushR32(operation) => operation.encode(encoder),
            Self::PushImm32(operation) => operation.encode(encoder),
            Self::PushImm8(operation) => operation.encode(encoder),
            Self::AddRm32Imm32(operation) => operation.encode(encoder),
            Self::OrRm32Imm32(operation) => operation.encode(encoder),
            Self::AdcRm32Imm32(operation) => operation.encode(encoder),
            Self::SbbRm32Imm32(operation) => operation.encode(encoder),
            Self::AndRm32Imm32(operation) => operation.encode(encoder),
            Self::SubRm32Imm32(operation) => operation.encode(encoder),
            Self::XorRm32Imm32(operation) => operation.encode(encoder),
            Self::CmpRm32Imm32(operation) => operation.encode(encoder),
            Self::AddRm32Imm8(operation) => operation.encode(encoder),
            Self::OrRm32Imm8(operation) => operation.encode(encoder),
            Self::AdcRm32Imm8(operation) => operation.encode(encoder),
            Self::SbbRm32Imm8(operation) => operation.encode(encoder),
            Self::AndRm32Imm8(operation) => operation.encode(encoder),
            Self::SubRm32Imm8(operation) => operation.encode(encoder),
            Self::XorRm32Imm8(operation) => operation.encode(encoder),
            Self::CmpRm32Imm8(operation) => operation.encode(encoder),
            Self::MovRm32R32(operation) => operation.encode(encoder),
            Self::MovR8Rm8(operation) => operation.encode(encoder),
            Self::MovR32Rm32(operation) => operation.encode(encoder),
            Self::MovEaxMoffs32(operation) => operation.encode(encoder),
            Self::RolRm32Imm8(operation) => operation.encode(encoder),
            Self::RorRm32Imm8(operation) => operation.encode(encoder),
            Self::RclRm32Imm8(operation) => operation.encode(encoder),
            Self::RcrRm32Imm8(operation) => operation.encode(encoder),
            Self::ShlRm32Imm8(operation) => operation.encode(encoder),
            Self::ShrRm32Imm8(operation) => operation.encode(encoder),
            Self::SarRm32Imm8(operation) => operation.encode(encoder),
            Self::CallRm32(operation) => operation.encode(encoder),
        }
    }
}

impl Decode for Operation {
    type Context<'a> = &'a Prefixes;
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        match decoder.read_u8()? {
            0x33 => Ok(Self::XorR32Rm32(XorR32Rm32::new(Rm32::decode(decoder)?))),
            byte @ 0x50..0x58 => Ok(Self::PushR32(PushR32::new(Register32::from_index(
                byte - 0x50,
            )))),
            0x68 => Ok(Self::PushImm32(PushImm32::new(Immediate32::decode(
                decoder,
            )?))),
            0x6a => Ok(Self::PushImm8(PushImm8::new(Immediate8::decode(decoder)?))),
            0x81 => {
                let operand = Rm32::decode(decoder)?;
                let immediate = Immediate32::decode(decoder)?;

                match operand.opcode_extension() {
                    0 => Ok(Self::AddRm32Imm32(AddRm32Imm32::new(operand, immediate))),
                    1 => Ok(Self::OrRm32Imm32(OrRm32Imm32::new(operand, immediate))),
                    2 => Ok(Self::AdcRm32Imm32(AdcRm32Imm32::new(operand, immediate))),
                    3 => Ok(Self::SbbRm32Imm32(SbbRm32Imm32::new(operand, immediate))),
                    4 => Ok(Self::AndRm32Imm32(AndRm32Imm32::new(operand, immediate))),
                    5 => Ok(Self::SubRm32Imm32(SubRm32Imm32::new(operand, immediate))),
                    6 => Ok(Self::XorRm32Imm32(XorRm32Imm32::new(operand, immediate))),
                    7 => Ok(Self::CmpRm32Imm32(CmpRm32Imm32::new(operand, immediate))),
                    _ => unreachable!("ModR/M opcode extension is three bits"),
                }
            }
            0x83 => {
                let operand = Rm32::decode(decoder)?;
                let immediate = Immediate8::decode(decoder)?;

                match operand.opcode_extension() {
                    0 => Ok(Self::AddRm32Imm8(AddRm32Imm8::new(operand, immediate))),
                    1 => Ok(Self::OrRm32Imm8(OrRm32Imm8::new(operand, immediate))),
                    2 => Ok(Self::AdcRm32Imm8(AdcRm32Imm8::new(operand, immediate))),
                    3 => Ok(Self::SbbRm32Imm8(SbbRm32Imm8::new(operand, immediate))),
                    4 => Ok(Self::AndRm32Imm8(AndRm32Imm8::new(operand, immediate))),
                    5 => Ok(Self::SubRm32Imm8(SubRm32Imm8::new(operand, immediate))),
                    6 => Ok(Self::XorRm32Imm8(XorRm32Imm8::new(operand, immediate))),
                    7 => Ok(Self::CmpRm32Imm8(CmpRm32Imm8::new(operand, immediate))),
                    _ => unreachable!("ModR/M opcode extension is three bits"),
                }
            }
            0x89 => Ok(Self::MovRm32R32(MovRm32R32::new(Rm32::decode(decoder)?))),
            0x8a => Ok(Self::MovR8Rm8(MovR8Rm8::new(Rm32::decode(decoder)?))),
            0x8b => Ok(Self::MovR32Rm32(MovR32Rm32::new(Rm32::decode(decoder)?))),
            0xa1 => Ok(Self::MovEaxMoffs32(MovEaxMoffs32::new(Address::decode(
                decoder,
            )?))),
            0xc1 => {
                let operand = Rm32::decode(decoder)?;
                let count = decoder.read_u8()?;

                match operand.opcode_extension() {
                    0 => Ok(Self::RolRm32Imm8(RolRm32Imm8::new(operand, count))),
                    1 => Ok(Self::RorRm32Imm8(RorRm32Imm8::new(operand, count))),
                    2 => Ok(Self::RclRm32Imm8(RclRm32Imm8::new(operand, count))),
                    3 => Ok(Self::RcrRm32Imm8(RcrRm32Imm8::new(operand, count))),
                    4 => Ok(Self::ShlRm32Imm8(ShlRm32Imm8::new(operand, count))),
                    5 => Ok(Self::ShrRm32Imm8(ShrRm32Imm8::new(operand, count))),
                    7 => Ok(Self::SarRm32Imm8(SarRm32Imm8::new(operand, count))),
                    extension => Err(Error::InvalidOpcodeExtension {
                        opcode: 0xc1,
                        extension,
                    }),
                }
            }
            0xff => {
                let operand = Rm32::decode(decoder)?;

                match operand.opcode_extension() {
                    2 => Ok(Self::CallRm32(CallRm32::new(operand))),
                    extension => Err(Error::InvalidOpcodeExtension {
                        opcode: 0xff,
                        extension,
                    }),
                }
            }
            opcode => Err(Error::InvalidOpcode(opcode)),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}
