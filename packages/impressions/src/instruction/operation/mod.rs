//! The x86 instruction operation.

mod error;

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use super::Mnemonic;
use super::operand::register::Register32;

pub use self::error::Error;

/// An x86 instruction operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Push r32.
    ///
    /// `50+rd => push r32`
    PushR32(Register32),
}

impl Operation {
    /// Gets the mnemonic of the operation.
    pub const fn mnemonic(&self) -> Mnemonic {
        match self {
            Self::PushR32(_) => Mnemonic::Push,
        }
    }
}

impl Extent for Operation {
    fn size(&self) -> Size {
        match self {
            Self::PushR32(_) => Size::new_valid(1),
        }
    }
}

impl Encode for Operation {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        match self {
            Self::PushR32(reg) => encoder.write_u8(0x50 | reg.index()),
        }
    }
}

impl Decode for Operation {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        match decoder.read_u8()? {
            byte @ 0x50..0x58 => Ok(Self::PushR32(Register32::from_index(byte - 0x50))),
            opcode => Err(Error::InvalidOpcode(opcode)),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PushR32(reg) => write!(f, "{} {reg}", self.mnemonic()),
        }
    }
}
