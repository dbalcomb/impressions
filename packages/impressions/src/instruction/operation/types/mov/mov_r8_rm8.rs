use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::memory::Memory32;
use crate::instruction::operand::register::Register8;
use crate::instruction::operation::encoding::Rm32;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Move an 8-bit register or memory value into an 8-bit register.
///
/// `8A /r => mov r8, r/m8`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovR8Rm8 {
    operand: Rm32,
}

impl MovR8Rm8 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(operand: Rm32) -> Self {
        Self { operand }
    }
}

impl MovR8Rm8 {
    /// Gets the source register, if the source is not memory.
    pub const fn source_register(&self) -> Option<Register8> {
        match self.operand.rm_register() {
            Some(register) => Some(Register8::from_index(register.index())),
            None => None,
        }
    }

    /// Gets the source address, if the source is memory.
    pub const fn source_memory(&self) -> Option<Memory32> {
        self.operand.memory()
    }

    /// Gets the destination register.
    pub const fn destination_register(&self) -> Register8 {
        Register8::from_index(self.operand.register().index())
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Mov
    }
}

impl MovR8Rm8 {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} ", self.mnemonic())?;
        write!(f, "{}, ", self.destination_register())?;

        if let Some(register) = self.source_register() {
            return Display::fmt(&register, f);
        }

        if prefixes.has_fs_override() {
            f.write_str("fs:")?;
        }

        Display::fmt(&self.operand, f)
    }
}

impl Extent for MovR8Rm8 {
    fn size(&self) -> Size {
        Size::new_valid(1 + self.operand.size().get())
    }
}

impl Encode for MovR8Rm8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x8a)?;
        self.operand.encode(encoder)
    }
}

impl Display for MovR8Rm8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefixes(f, &Prefixes::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::register::Register8;
    use crate::instruction::operation::{Error, Operation};
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    #[test]
    fn supports_all_register_forms() {
        for destination in 0..8 {
            for source in 0..8 {
                let bytes = [0x8a, 0xc0 | destination << 3 | source];
                let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
                let Operation::MovR8Rm8(operation) = operation else {
                    panic!("expected MOV");
                };
                let destination = Register8::from_index(destination);
                let source = Register8::from_index(source);

                assert_eq!(operation.destination_register(), destination);
                assert_eq!(operation.source_register(), Some(source));
                assert_eq!(operation.source_memory(), None);
                assert_eq!(operation.mnemonic(), Mnemonic::Mov);
                assert_eq!(
                    operation.to_string(),
                    format!("mov {destination}, {source}")
                );
                assert_eq!(operation.size(), 2);

                let mut encoded = Vec::new();
                operation.encode(&mut encoded).unwrap();
                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn supports_memory_forms() {
        for (bytes, memory) in [
            (&b"\x8a\x23"[..], "[ebx]"),
            (&b"\x8a\x63\xf0"[..], "[ebx-16]"),
            (&b"\x8a\xa3\xf0\xff\xff\xff"[..], "[ebx-16]"),
            (&b"\x8a\x25\x38\x81\x5d\x00"[..], "[0x005d8138]"),
            (&b"\x8a\x24\x24"[..], "[esp]"),
            (&b"\x8a\x65\x00"[..], "[ebp]"),
            (&b"\x8a\x24\x8b"[..], "[ebx+ecx*4]"),
            (&b"\x8a\x64\x8b\xf0"[..], "[ebx+ecx*4-16]"),
            (&b"\x8a\xa4\x8b\xf0\xff\xff\xff"[..], "[ebx+ecx*4-16]"),
            (&b"\x8a\x24\x8d\x10\x00\x00\x00"[..], "[ecx*4+16]"),
            (&b"\x8a\x24\xe5\x38\x81\x5d\x00"[..], "[0x005d8138]"),
            (&b"\x8a\x63\x00"[..], "[ebx]"),
            (&b"\x8a\xa3\x00\x00\x00\x00"[..], "[ebx]"),
            (&b"\x8a\x24\x0b"[..], "[ebx+ecx*1]"),
            (&b"\x8a\x24\x4b"[..], "[ebx+ecx*2]"),
            (&b"\x8a\x24\xcb"[..], "[ebx+ecx*8]"),
        ] {
            let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
            let Operation::MovR8Rm8(operation) = operation else {
                panic!("expected MOV");
            };

            assert_eq!(operation.destination_register(), Register8::Ah);
            assert_eq!(operation.source_register(), None);
            assert_eq!(operation.source_memory().unwrap().to_string(), memory);
            assert_eq!(operation.to_string(), format!("mov ah, {memory}"));
            assert_eq!(operation.size(), bytes.len() as u64);

            let mut encoded = Vec::new();

            operation.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }
    }

    #[test]
    fn rejects_truncation() {
        for bytes in [
            &b"\x8a\xd4"[..],
            &b"\x8a\x25\x38\x81\x5d\x00"[..],
            &b"\x8a\xa4\x8b\xf0\xff\xff\xff"[..],
        ] {
            for length in 1..bytes.len() {
                assert!(matches!(
                    Operation::decode_with(&mut &bytes[..length], &Prefixes::new()),
                    Err(Error::Decode(_)),
                ));
            }
        }
    }
}
