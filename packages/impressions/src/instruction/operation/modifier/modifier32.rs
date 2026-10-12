use crate::instruction::operand::displacement::DisplacementSize;
use crate::instruction::operand::memory::Scale;
use crate::instruction::operand::register::Register32;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// The ModR/M and SIB byte modifiers for x86 instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modifier32 {
    modrm: u8,
    sib: Option<u8>,
}

impl Modifier32 {
    /// Gets the raw ModR/M byte.
    pub const fn modrm(self) -> u8 {
        self.modrm
    }

    /// Gets the ModR/M `mod` field.
    pub const fn mode(self) -> u8 {
        self.modrm >> 6
    }

    /// Gets the ModR/M `reg` field.
    pub const fn reg_index(self) -> u8 {
        (self.modrm >> 3) & 0x07
    }

    /// Gets the ModR/M `reg` field as a 32-bit register.
    pub const fn reg(self) -> Register32 {
        Register32::from_index(self.reg_index())
    }

    /// Gets the ModR/M `r/m` field.
    pub const fn rm(self) -> Register32 {
        Register32::from_index(self.modrm & 0x07)
    }

    /// Gets the SIB byte, if this addressing form has one.
    pub const fn sib(self) -> Option<u8> {
        self.sib
    }

    /// Gets the SIB `scale` field, if this addressing form has a SIB byte.
    pub const fn scale(self) -> Option<Scale> {
        match self.sib {
            Some(sib) => Some(Scale::from_index(sib >> 6)),
            None => None,
        }
    }

    /// Gets the SIB `index` field, if this addressing form has a SIB byte.
    pub const fn index(self) -> Option<Register32> {
        match self.sib {
            Some(sib) if (sib >> 3) & 0x07 != Register32::Esp.index() => {
                Some(Register32::from_index((sib >> 3) & 0x07))
            }
            _ => None,
        }
    }

    /// Gets the SIB `base` field, if this addressing form has a SIB byte.
    pub const fn base(self) -> Option<Register32> {
        match self.sib {
            Some(sib) if self.mode() == 0 && sib & 0x07 == Register32::Ebp.index() => None,
            Some(sib) => Some(Register32::from_index(sib & 0x07)),
            None => None,
        }
    }

    /// Gets the effective base register, if the memory address has one.
    ///
    /// Returns `None` for register mode and addresses without a base.
    pub const fn memory_base(self) -> Option<Register32> {
        match self.mode() {
            3 => None,
            _ if self.sib.is_some() => self.base(),
            0 if matches!(self.rm(), Register32::Ebp) => None,
            _ => Some(self.rm()),
        }
    }

    /// Gets the displacement size.
    pub const fn displacement_size(self) -> Option<DisplacementSize> {
        match self.mode() {
            0 if self.memory_base().is_none() => Some(DisplacementSize::DWord),
            1 => Some(DisplacementSize::Byte),
            2 => Some(DisplacementSize::DWord),
            _ => None,
        }
    }
}

impl Extent for Modifier32 {
    fn size(&self) -> Size {
        Size::new_valid(match self.sib {
            Some(_) => 2,
            None => 1,
        })
    }
}

impl Encode for Modifier32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(self.modrm)?;

        if let Some(sib) = self.sib {
            encoder.write_u8(sib)?;
        }

        Ok(())
    }
}

impl Decode for Modifier32 {
    type Context<'a> = ();
    type Error = DecodeError;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let modrm = decoder.read_u8()?;
        let has_sib = modrm & 0xc0 != 0xc0 && modrm & 0x07 == Register32::Esp.index();
        let sib = match has_sib {
            true => Some(decoder.read_u8()?),
            false => None,
        };

        Ok(Self { modrm, sib })
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::operand::displacement::DisplacementSize;
    use crate::instruction::operand::memory::Scale;
    use crate::instruction::operand::register::Register32;
    use crate::memory::region::ops::decode::{Decode, Error};

    use crate::memory::extent::Extent;
    use crate::memory::region::ops::encode::Encode;

    use super::Modifier32;

    #[test]
    fn round_trips_every_modrm_and_sib_encoding() {
        for modrm in 0..=u8::MAX {
            let mode = modrm >> 6;
            let rm = modrm & 7;
            let has_sib = mode != 3 && rm == 4;

            for sib in 0..if has_sib { 256 } else { 1 } {
                let mut bytes = vec![modrm];

                if has_sib {
                    bytes.push(sib as u8);
                }

                let mut decoder = bytes.as_slice();
                let modifier = Modifier32::decode(&mut decoder).unwrap();
                let mut encoded = Vec::new();

                modifier.encode(&mut encoded).unwrap();

                assert!(decoder.is_empty());
                assert_eq!(encoded, bytes);
                assert_eq!(modifier.size(), bytes.len() as u64);
                assert_eq!(modifier.modrm(), modrm);
                assert_eq!(modifier.mode(), mode);
                assert_eq!(modifier.reg().index(), (modrm >> 3) & 7);
                assert_eq!(modifier.rm().index(), rm);
                assert_eq!(modifier.sib(), if has_sib { Some(sib as u8) } else { None });
            }
        }
    }

    #[test]
    fn decodes_modrm_and_sib_together() {
        let mut bytes = &b"\x44\x94\x7f\x42"[..];
        let modifier = Modifier32::decode(&mut bytes).unwrap();

        assert_eq!(modifier.reg(), Register32::Eax);
        assert_eq!(modifier.rm(), Register32::Esp);
        assert_eq!(modifier.base(), Some(Register32::Esp));
        assert_eq!(modifier.index(), Some(Register32::Edx));
        assert_eq!(modifier.scale(), Some(Scale::Four));
        assert_eq!(modifier.displacement_size(), Some(DisplacementSize::Byte));
        assert_eq!(bytes, b"\x7f\x42");
    }

    #[test]
    fn selects_memory_base_for_sib_and_non_sib_forms() {
        for (bytes, base) in [
            (&b"\xc3"[..], None),
            (&b"\x03"[..], Some(Register32::Ebx)),
            (&b"\x05"[..], None),
            (&b"\x45"[..], Some(Register32::Ebp)),
            (&b"\x85"[..], Some(Register32::Ebp)),
            (&b"\x04\x24"[..], Some(Register32::Esp)),
            (&b"\x04\x25"[..], None),
            (&b"\x44\x25"[..], Some(Register32::Ebp)),
            (&b"\x84\x25"[..], Some(Register32::Ebp)),
        ] {
            let modifier = Modifier32::decode(&mut &bytes[..]).unwrap();

            assert_eq!(modifier.memory_base(), base);
        }
    }

    #[test]
    fn distinguishes_register_and_absolute_displacements() {
        for (bytes, size) in [
            (&b"\xc5"[..], None),
            (&b"\x05"[..], Some(DisplacementSize::DWord)),
            (&b"\x04\x25"[..], Some(DisplacementSize::DWord)),
            (&b"\x44\x25"[..], Some(DisplacementSize::Byte)),
        ] {
            let modifier = Modifier32::decode(&mut &bytes[..]).unwrap();

            assert_eq!(modifier.displacement_size(), size);
        }
    }

    #[test]
    fn retains_ignored_sib_scale_and_reports_truncated_sib() {
        let modifier = Modifier32::decode(&mut &b"\x04\xe3"[..]).unwrap();

        assert_eq!(modifier.index(), None);
        assert_eq!(modifier.scale(), Some(Scale::Eight));

        let mut encoded = Vec::new();

        modifier.encode(&mut encoded).unwrap();

        assert_eq!(encoded, b"\x04\xe3");
        assert_eq!(
            Modifier32::decode(&mut &b"\x04"[..]),
            Err(Error::InsufficientBytes {
                requested: 1,
                available: 0,
            })
        );
    }

    #[test]
    fn register_mode_leaves_potential_sib_byte_unread() {
        let mut bytes = &b"\xc4\xe3"[..];
        let modifier = Modifier32::decode(&mut bytes).unwrap();

        assert_eq!(modifier.rm(), Register32::Esp);
        assert_eq!(modifier.sib(), None);
        assert_eq!(bytes, b"\xe3");
    }
}
