use std::fmt::{self, Display};

use crate::instruction::operand::displacement::{
    Displacement, Displacement8, Displacement32, DisplacementSize,
};
use crate::instruction::operand::memory::{Memory32, ScaledIndex};
use crate::instruction::operand::register::Register32;
use crate::instruction::operation::modifier::{Modifier32, RegField};
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// A ModR/M operand with 32-bit addressing, retaining its modifier and displacement.
///
/// The `reg` field can encode a register operand or an opcode extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rm32 {
    modifier: Modifier32,
    displacement: Displacement32,
}

impl Rm32 {
    /// Gets the opcode extension in the ModR/M `reg` field.
    pub const fn opcode_extension(&self) -> RegField {
        self.modifier.reg_field()
    }

    /// Gets the register in the ModR/M `reg` field.
    pub const fn register(&self) -> Register32 {
        self.modifier.reg()
    }

    /// Gets the r/m register for register-mode encodings.
    pub const fn rm_register(&self) -> Option<Register32> {
        if self.modifier.mode() == 3 {
            Some(self.modifier.rm())
        } else {
            None
        }
    }

    /// Gets a displacement when the modifier encodes one.
    pub const fn displacement(&self) -> Option<Displacement> {
        match self.modifier.displacement_size() {
            None => None,
            Some(DisplacementSize::Byte) => Some(Displacement::Byte(Displacement8::new(
                self.displacement.value() as i8,
            ))),
            Some(DisplacementSize::DWord) => Some(Displacement::DWord(self.displacement)),
        }
    }

    /// Gets a simplified memory address for memory-mode encodings.
    pub const fn memory(&self) -> Option<Memory32> {
        if self.modifier.mode() == 3 {
            return None;
        }

        let base = self.modifier.memory_base();
        let index = match (self.modifier.index(), self.modifier.scale()) {
            (Some(register), Some(scale)) => Some(ScaledIndex::new(register, scale)),
            _ => None,
        };

        Memory32::new(base, index, self.displacement)
    }
}

impl Extent for Rm32 {
    fn size(&self) -> Size {
        let displacement_size = match self.modifier.displacement_size() {
            None => 0,
            Some(DisplacementSize::Byte) => 1,
            Some(DisplacementSize::DWord) => 4,
        };

        Size::new_valid(self.modifier.size().get() + displacement_size)
    }
}

impl Encode for Rm32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        self.modifier.encode(encoder)?;

        match self.displacement() {
            None => Ok(()),
            Some(Displacement::Byte(value)) => value.encode(encoder),
            Some(Displacement::DWord(value)) => value.encode(encoder),
        }
    }
}

impl Decode for Rm32 {
    type Context<'a> = ();
    type Error = DecodeError;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let modifier = Modifier32::decode(decoder)?;
        let displacement = match modifier.displacement_size() {
            None => Displacement32::default(),
            Some(DisplacementSize::Byte) => {
                Displacement32::new(Displacement8::decode(decoder)?.value() as i32)
            }
            Some(DisplacementSize::DWord) => Displacement32::decode(decoder)?,
        };

        Ok(Self {
            modifier,
            displacement,
        })
    }
}

impl Display for Rm32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.memory() {
            Some(memory) => Display::fmt(&memory, f),
            None => Display::fmt(&self.modifier.rm(), f),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::operand::displacement::{
        Displacement, Displacement8, Displacement32, DisplacementSize,
    };
    use crate::instruction::operand::memory::{Scale, ScaledIndex};
    use crate::instruction::operand::register::Register32;
    use crate::instruction::operation::modifier::{Modifier32, RegField};
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::{Decode, Error as DecodeError};
    use crate::memory::region::ops::encode::Encode;

    use super::Rm32;

    impl Rm32 {
        fn new(modifier: Modifier32, displacement: Displacement32) -> Self {
            let displacement = match modifier.displacement_size() {
                None => Displacement32::default(),
                Some(DisplacementSize::Byte) => {
                    assert!(
                        i8::try_from(displacement.value()).is_ok(),
                        "displacement {} does not fit in a signed byte",
                        displacement.value()
                    );

                    displacement
                }
                Some(DisplacementSize::DWord) => displacement,
            };

            Self {
                modifier,
                displacement,
            }
        }
    }

    #[test]
    fn supports_register_and_memory_forms() {
        let byte = Some(Displacement::Byte(Displacement8::new(-16)));
        let dword = Some(Displacement::DWord(Displacement32::new(-16)));
        let index = Some(ScaledIndex::new(Register32::Ecx, Scale::Four));

        for (modifier_bytes, value, displacement, memory, bytes, display) in [
            (&b"\xc1"[..], 0, None, None, &b"\xc1"[..], "ecx"),
            (
                &b"\x03"[..],
                0,
                None,
                Some((Some(Register32::Ebx), None)),
                &b"\x03"[..],
                "[ebx]",
            ),
            (
                &b"\x43"[..],
                -16,
                byte,
                Some((Some(Register32::Ebx), None)),
                &b"\x43\xf0"[..],
                "[ebx-16]",
            ),
            (
                &b"\x83"[..],
                -16,
                dword,
                Some((Some(Register32::Ebx), None)),
                &b"\x83\xf0\xff\xff\xff"[..],
                "[ebx-16]",
            ),
            (
                &b"\x05"[..],
                -16,
                dword,
                Some((None, None)),
                &b"\x05\xf0\xff\xff\xff"[..],
                "[0xfffffff0]",
            ),
            (
                &b"\x04\x8b"[..],
                0,
                None,
                Some((Some(Register32::Ebx), index)),
                &b"\x04\x8b"[..],
                "[ebx+ecx*4]",
            ),
            (
                &b"\x44\x8b"[..],
                -16,
                byte,
                Some((Some(Register32::Ebx), index)),
                &b"\x44\x8b\xf0"[..],
                "[ebx+ecx*4-16]",
            ),
            (
                &b"\x84\x8b"[..],
                -16,
                dword,
                Some((Some(Register32::Ebx), index)),
                &b"\x84\x8b\xf0\xff\xff\xff"[..],
                "[ebx+ecx*4-16]",
            ),
            (
                &b"\x04\xe3"[..],
                0,
                None,
                Some((Some(Register32::Ebx), None)),
                &b"\x04\xe3"[..],
                "[ebx]",
            ),
            (
                &b"\x44\x24"[..],
                -16,
                byte,
                Some((Some(Register32::Esp), None)),
                &b"\x44\x24\xf0"[..],
                "[esp-16]",
            ),
            (
                &b"\x84\x24"[..],
                -16,
                dword,
                Some((Some(Register32::Esp), None)),
                &b"\x84\x24\xf0\xff\xff\xff"[..],
                "[esp-16]",
            ),
            (
                &b"\x04\x8d"[..],
                -16,
                dword,
                Some((None, index)),
                &b"\x04\x8d\xf0\xff\xff\xff"[..],
                "[ecx*4-16]",
            ),
            (
                &b"\x04\xe5"[..],
                -16,
                dword,
                Some((None, None)),
                &b"\x04\xe5\xf0\xff\xff\xff"[..],
                "[0xfffffff0]",
            ),
        ] {
            let modifier = Modifier32::decode(&mut &modifier_bytes[..]).unwrap();
            let operands = Rm32::new(modifier, Displacement32::new(value));
            let mut encoder = Vec::new();

            operands.encode(&mut encoder).unwrap();

            assert_eq!(operands.modifier, modifier);
            assert_eq!(operands.register(), Register32::Eax);
            assert_eq!(operands.opcode_extension(), RegField::Zero);
            assert_eq!(operands.displacement(), displacement);
            assert_eq!(
                operands.rm_register(),
                if memory.is_none() {
                    Some(Register32::Ecx)
                } else {
                    None
                }
            );
            assert_eq!(operands.to_string(), display);
            assert_eq!(operands.size(), bytes.len() as u64);
            assert_eq!(encoder, bytes);

            match (operands.memory(), memory) {
                (Some(actual), Some((base, index))) => {
                    assert_eq!(actual.base(), base);
                    assert_eq!(actual.index(), index);
                    assert_eq!(actual.displacement(), Displacement32::new(value));
                }
                (None, None) => {}
                _ => panic!("unexpected memory view for {display}"),
            }

            let mut decoder = bytes;

            assert_eq!(Rm32::decode(&mut decoder).unwrap(), operands);
            assert!(decoder.is_empty());
        }
    }

    #[test]
    fn extracts_sign_extended_byte_and_semantic_memory() {
        let operands = Rm32::decode(&mut &b"\x44\x8b\xf0"[..]).unwrap();

        assert_eq!(
            operands
                .displacement()
                .unwrap()
                .displacement8()
                .unwrap()
                .value(),
            -16
        );
        assert_eq!(operands.memory().unwrap().to_string(), "[ebx+ecx*4-16]");

        let memory = operands.memory().unwrap();

        assert_eq!(memory.displacement().value(), -16);
    }

    #[test]
    fn ignores_unused_storage_and_rejects_invalid_encoding() {
        let operands = Rm32::new(
            Modifier32::decode(&mut &b"\xc1"[..]).unwrap(),
            Displacement32::new(123),
        );

        assert_eq!(operands.displacement(), None);
        assert_eq!(operands.memory(), None);

        let mut encoder = Vec::new();

        operands.encode(&mut encoder).unwrap();

        assert_eq!(encoder, b"\xc1");
    }

    #[test]
    fn keeps_encoded_zero_width_but_returns_equivalent_memory_views() {
        let mut encodings = Vec::new();

        for bytes in [
            &b"\x03"[..],
            &b"\x43\x00"[..],
            &b"\x83\x00\x00\x00\x00"[..],
            &b"\x04\xe3"[..],
        ] {
            encodings.push(Rm32::decode(&mut &bytes[..]).unwrap());
        }

        for operands in &encodings {
            assert_eq!(operands.memory(), encodings[0].memory());
            assert_eq!(operands.memory().unwrap().to_string(), "[ebx]");
        }

        assert_eq!(encodings[0].displacement(), None);
        assert_eq!(
            encodings[1].displacement(),
            Some(Displacement::Byte(Displacement8::new(0)))
        );
        assert_eq!(
            encodings[2].displacement(),
            Some(Displacement::DWord(Displacement32::new(0)))
        );
        assert_eq!(encodings[3].displacement(), None);
        assert_ne!(encodings[0], encodings[3]);
    }

    #[test]
    fn preserves_all_signed_byte_values_and_dword_extremes() {
        for value in i8::MIN..=i8::MAX {
            let bytes = [0x43, value as u8];
            let operands = Rm32::decode(&mut &bytes[..]).unwrap();

            assert_eq!(
                operands.displacement(),
                Some(Displacement::Byte(Displacement8::new(value)))
            );
            assert_eq!(
                operands.memory().unwrap().displacement().value(),
                i32::from(value)
            );
            assert_eq!(
                Rm32::new(operands.modifier, Displacement32::new(i32::from(value))),
                operands,
            );
        }

        for value in [i32::MIN, i32::MAX, -1, 0] {
            let operands = Rm32::new(
                Modifier32::decode(&mut &b"\x83"[..]).unwrap(),
                Displacement32::new(value),
            );
            let mut encoder = Vec::new();

            operands.encode(&mut encoder).unwrap();

            assert_eq!(Rm32::decode(&mut encoder.as_slice()).unwrap(), operands);
        }
    }

    #[test]
    fn derives_direct_and_no_base_sib_addresses() {
        for bytes in [
            &b"\x05\x00\x00\x00\x80"[..],
            &b"\x04\xe5\x00\x00\x00\x80"[..],
        ] {
            let operands = Rm32::decode(&mut &bytes[..]).unwrap();
            let memory = operands.memory().unwrap();

            assert_eq!(memory.base(), None);
            assert_eq!(memory.index(), None);
            assert_eq!(memory.to_string(), "[0x80000000]");
        }

        let operands = Rm32::decode(&mut &b"\x04\x8d\x10\x00\x00\x00"[..]).unwrap();

        assert_eq!(operands.memory().unwrap().to_string(), "[ecx*4+16]");

        let operands = Rm32::decode(&mut &b"\x45\x00"[..]).unwrap();

        assert_eq!(operands.memory().unwrap().base(), Some(Register32::Ebp));
    }

    #[test]
    fn rejects_truncated_input() {
        for bytes in [
            &b""[..],
            &b"\x04"[..],
            &b"\x43"[..],
            &b"\x83\x01\x02\x03"[..],
            &b"\x04\x25\x01\x02\x03"[..],
        ] {
            assert!(matches!(
                Rm32::decode(&mut &bytes[..]),
                Err(DecodeError::InsufficientBytes { .. }),
            ));
        }
    }

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

            assert_eq!(operand.opcode_extension(), RegField::Five);
            assert_eq!(operand.rm_register(), register);
            assert_eq!(operand.memory().is_some(), register.is_none());
            assert_eq!(operand.displacement(), displacement);
            assert_eq!(operand.to_string(), display);
            assert_eq!(operand.size(), bytes.len() as u64);

            let mut encoded = Vec::new();

            operand.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }
    }
}
