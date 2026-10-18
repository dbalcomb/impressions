use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::register::Register32;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use crate::instruction::operation::encoding::Rm32;

/// Move a 32-bit register value into a 32-bit register or memory location.
///
/// `89 /r => mov r/m32, r32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovRm32R32 {
    operand: Rm32,
}

impl MovRm32R32 {
    /// Creates the operation from its encoded operands.
    pub const fn new(operand: Rm32) -> Self {
        Self { operand }
    }

    /// Gets the shared ModR/M encoding.
    pub const fn operand(&self) -> Rm32 {
        self.operand
    }

    /// Gets the explicit register operand in the ModR/M `reg` field.
    pub const fn register(&self) -> Register32 {
        self.operand.register()
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Mov
    }
}

impl MovRm32R32 {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} ", self.mnemonic())?;

        if prefixes.has_fs_override() && self.operand.memory().is_some() {
            f.write_str("fs:")?;
        }

        write!(f, "{}, {}", self.operand, self.register())
    }
}

impl Extent for MovRm32R32 {
    fn size(&self) -> Size {
        Size::new_valid(1 + self.operand.size().get())
    }
}

impl Encode for MovRm32R32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x89)?;
        self.operand.encode(encoder)
    }
}

impl Display for MovRm32R32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operation::Operation;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn supports_all_register_and_memory_forms() {
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
}
