use std::fmt::{self, Display};

use crate::instruction::operand::displacement::{Displacement, Displacement8, Displacement32};
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register32;
use crate::instruction::operation::modifier::Modifier32;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use super::R32Rm32;

/// A 32-bit register or memory operand paired with a 32-bit register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rm32R32(R32Rm32);

impl Rm32R32 {
    /// Gets the original ModR/M and optional SIB encoding.
    pub const fn modifier(&self) -> Modifier32 {
        self.0.modifier()
    }

    /// Gets the register in the ModR/M `reg` field.
    pub const fn register(&self) -> Register32 {
        self.0.register()
    }

    /// Gets the r/m register for register-mode encodings.
    pub const fn rm_register(&self) -> Option<Register32> {
        self.0.rm_register()
    }

    /// Gets a displacement when the modifier encodes one.
    pub const fn displacement(&self) -> Option<Displacement> {
        self.0.displacement()
    }

    /// Gets the displacement when it is encoded as a byte.
    pub const fn displacement8(&self) -> Option<Displacement8> {
        self.0.displacement8()
    }

    /// Gets the displacement when it is encoded as a dword.
    pub const fn displacement32(&self) -> Option<Displacement32> {
        self.0.displacement32()
    }

    /// Gets a simplified memory address for memory-mode encodings.
    pub const fn memory(&self) -> Option<Memory32> {
        self.0.memory()
    }
}

impl Extent for Rm32R32 {
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl Encode for Rm32R32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl Decode for Rm32R32 {
    type Context<'a> = ();
    type Error = DecodeError;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        R32Rm32::decode(decoder).map(Self)
    }
}

impl Display for Rm32R32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.memory() {
            Some(memory) => Display::fmt(&memory, f)?,
            None => Display::fmt(&self.modifier().rm(), f)?,
        }

        write!(f, ", {}", self.register())
    }
}
