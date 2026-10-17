use std::fmt::{self, Display};

use crate::instruction::operand::displacement::{Displacement, Displacement8, Displacement32};
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register32;
use crate::instruction::operation::modifier::Modifier32;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use super::R32Rm32;

/// A 32-bit register or memory operand, retaining its modifier and
/// displacement.
///
/// The ModR/M `reg` field is available as an opcode extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rm32(R32Rm32);

impl Rm32 {
    /// Gets the original ModR/M and optional SIB encoding.
    pub const fn modifier(&self) -> Modifier32 {
        self.0.modifier()
    }

    /// Gets the opcode extension in the ModR/M `reg` field.
    pub const fn opcode_extension(&self) -> u8 {
        self.modifier().reg_index()
    }

    /// Gets the destination register for register-mode encodings.
    pub const fn rm_register(&self) -> Option<Register32> {
        self.0.rm_register()
    }

    /// Gets a simplified memory address for memory-mode encodings.
    pub const fn memory(&self) -> Option<Memory32> {
        self.0.memory()
    }

    /// Gets the encoded displacement, retaining its width.
    pub const fn displacement(&self) -> Option<Displacement> {
        self.0.displacement()
    }

    /// Gets the displacement when encoded as a byte.
    pub const fn displacement8(&self) -> Option<Displacement8> {
        self.0.displacement8()
    }

    /// Gets the displacement when encoded as a dword.
    pub const fn displacement32(&self) -> Option<Displacement32> {
        self.0.displacement32()
    }
}

impl Extent for Rm32 {
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl Encode for Rm32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl Decode for Rm32 {
    type Context<'a> = ();
    type Error = DecodeError;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        R32Rm32::decode(decoder).map(Self)
    }
}

impl Display for Rm32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.memory() {
            Some(memory) => Display::fmt(&memory, f),
            None => Display::fmt(&self.modifier().rm(), f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_register_and_displaced_memory_without_an_immediate() {
        for (bytes, register, displacement, display) in [
            (&b"\xec"[..], Some(Register32::Esp), None, "esp"),
            (&b"\x2b"[..], None, None, "[ebx]"),
            (
                &b"\x6c\x8b\xf0"[..],
                None,
                Some(Displacement::Byte(Displacement8::new(-16))),
                "[ebx+ecx*4-16]",
            ),
            (
                &b"\xac\x8b\xf0\xff\xff\xff"[..],
                None,
                Some(Displacement::DWord(Displacement32::new(-16))),
                "[ebx+ecx*4-16]",
            ),
        ] {
            let operand = Rm32::decode(&mut &bytes[..]).unwrap();

            assert_eq!(operand.opcode_extension(), 5);
            assert_eq!(operand.rm_register(), register);
            assert_eq!(operand.memory().is_some(), register.is_none());
            assert_eq!(operand.displacement(), displacement);
            assert_eq!(
                operand.displacement8(),
                displacement.and_then(Displacement::displacement8)
            );
            assert_eq!(
                operand.displacement32(),
                displacement.and_then(Displacement::displacement32)
            );
            assert_eq!(operand.to_string(), display);
            assert_eq!(operand.size(), bytes.len() as u64);

            let mut encoded = Vec::new();

            operand.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }
    }
}
