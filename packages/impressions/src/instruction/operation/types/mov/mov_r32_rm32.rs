use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register32;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use crate::instruction::operation::encoding::Rm32;

/// Move a 32-bit register or memory value into a 32-bit register.
///
/// `8B /r => mov r32, r/m32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovR32Rm32 {
    operand: Rm32,
}

impl MovR32Rm32 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(operand: Rm32) -> Self {
        Self { operand }
    }
}

impl MovR32Rm32 {
    /// Gets the source register, if the source is not memory.
    pub const fn source_register(&self) -> Option<Register32> {
        self.operand.rm_register()
    }

    /// Gets the source address, if the source is memory.
    pub const fn source_memory(&self) -> Option<Memory32> {
        self.operand.memory()
    }

    /// Gets the destination register.
    pub const fn destination_register(&self) -> Register32 {
        self.operand.register()
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Mov
    }
}

impl MovR32Rm32 {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} ", self.mnemonic())?;
        write!(f, "{}, ", self.destination_register())?;

        if prefixes.has_fs_override() && self.operand.memory().is_some() {
            f.write_str("fs:")?;
        }

        Display::fmt(&self.operand, f)
    }
}

impl Extent for MovR32Rm32 {
    fn size(&self) -> Size {
        Size::new_valid(1 + self.operand.size().get())
    }
}

impl Encode for MovR32Rm32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x8b)?;
        self.operand.encode(encoder)
    }
}

impl Display for MovR32Rm32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::register::Register32;
    use crate::instruction::operation::Operation;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn exposes_source_and_destination() {
        for destination in 0..8 {
            for source in 0..8 {
                let bytes = [0x8b, 0xc0 | destination << 3 | source];
                let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                let Operation::MovR32Rm32(operation) = operation else {
                    panic!("expected MOV");
                };

                assert_eq!(
                    operation.destination_register(),
                    Register32::from_index(destination)
                );
                assert_eq!(
                    operation.source_register(),
                    Some(Register32::from_index(source))
                );
                assert_eq!(operation.source_memory(), None);
            }
        }

        for (bytes, destination, memory) in [
            (&b"\x8b\x54\x8b\xf0"[..], Register32::Edx, "[ebx+ecx*4-16]"),
            (
                &b"\x8b\x25\x38\x81\x5d\x00"[..],
                Register32::Esp,
                "[0x005d8138]",
            ),
        ] {
            let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
            let Operation::MovR32Rm32(operation) = operation else {
                panic!("expected MOV");
            };

            assert_eq!(operation.destination_register(), destination);
            assert_eq!(operation.source_register(), None);
            assert_eq!(operation.source_memory().unwrap().to_string(), memory);
        }
    }

    #[test]
    fn supports_all_register_and_memory_forms() {
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
}
