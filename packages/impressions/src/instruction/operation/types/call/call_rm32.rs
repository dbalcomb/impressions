use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::prefix::Prefixes;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use crate::instruction::operation::encoding::Rm32;

/// Call a near procedure indirectly through r/m32.
///
/// `FF /2 => call r/m32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallRm32 {
    operand: Rm32,
}

impl CallRm32 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(operand: Rm32) -> Self {
        Self { operand }
    }
}

impl CallRm32 {
    /// Gets the shared ModR/M encoding.
    pub const fn operand(&self) -> Rm32 {
        self.operand
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Call
    }
}

impl CallRm32 {
    pub(in crate::instruction::operation) fn fmt_with_prefixes(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefixes: &Prefixes,
    ) -> fmt::Result {
        write!(f, "{} ", self.mnemonic())?;

        if self.operand.memory().is_some() {
            f.write_str("dword ptr ")?;
        }

        if prefixes.has_fs_override() && self.operand.memory().is_some() {
            f.write_str("fs:")?;
        }

        Display::fmt(&self.operand, f)
    }
}

impl Extent for CallRm32 {
    fn size(&self) -> Size {
        Size::new_valid(1 + self.operand.size().get())
    }
}

impl Encode for CallRm32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0xff)?;
        self.operand.encode(encoder)
    }
}

impl Display for CallRm32 {
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
    fn supports_all_register_and_memory_forms() {
        for (bytes, display) in [
            (
                &b"\xff\x15\x38\x81\x5d\x00"[..],
                "call dword ptr [0x005d8138]",
            ),
            (&b"\xff\x13"[..], "call dword ptr [ebx]"),
            (&b"\xff\x53\xf0"[..], "call dword ptr [ebx-16]"),
            (&b"\xff\x93\xf0\xff\xff\xff"[..], "call dword ptr [ebx-16]"),
            (&b"\xff\x14\x24"[..], "call dword ptr [esp]"),
            (&b"\xff\x55\x00"[..], "call dword ptr [ebp]"),
            (&b"\xff\x14\x8b"[..], "call dword ptr [ebx+ecx*4]"),
            (&b"\xff\x54\x8b\xf0"[..], "call dword ptr [ebx+ecx*4-16]"),
            (
                &b"\xff\x94\x8b\xf0\xff\xff\xff"[..],
                "call dword ptr [ebx+ecx*4-16]",
            ),
            (
                &b"\xff\x14\x8d\x10\x00\x00\x00"[..],
                "call dword ptr [ecx*4+16]",
            ),
            (
                &b"\xff\x14\xe5\x38\x81\x5d\x00"[..],
                "call dword ptr [0x005d8138]",
            ),
        ] {
            let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();

            assert_eq!(operation.mnemonic(), Mnemonic::Call);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), bytes.len() as u64);

            let Operation::CallRm32(operand) = &operation else {
                panic!("expected CALL");
            };

            assert_eq!(operand.operand().opcode_extension(), 2);
            assert!(operand.operand().memory().is_some());

            let mut encoded = Vec::new();

            operation.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }

        for index in 0..8 {
            let bytes = [0xff, 0xd0 | index];
            let operation = Operation::decode_with(&mut &bytes[..], &Prefixes::new()).unwrap();
            let register = Register32::from_index(index);

            assert_eq!(operation.to_string(), format!("call {register}"));
            assert_eq!(operation.size(), 2);

            let Operation::CallRm32(operand) = &operation else {
                panic!("expected CALL");
            };

            assert_eq!(operand.operand().rm_register(), Some(register));

            let mut encoded = Vec::new();

            operation.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }
    }

    #[test]
    fn rejects_truncation_and_unsupported_extensions() {
        for bytes in [&b"\xff\x15\x38\x81\x5d\x00"[..], &b"\xff\x54\x8b\xf0"[..]] {
            for length in 1..bytes.len() {
                assert!(matches!(
                    Operation::decode_with(&mut &bytes[..length], &Prefixes::new()),
                    Err(crate::instruction::operation::Error::Decode(_)),
                ));
            }
        }

        for extension in [0, 1, 3, 4, 5, 6, 7] {
            let bytes = [0xff, 0xc0 | extension << 3];

            assert_eq!(
                Operation::decode_with(&mut &bytes[..], &Prefixes::new()),
                Err(
                    crate::instruction::operation::Error::InvalidOpcodeExtension {
                        opcode: 0xff,
                        extension
                    }
                ),
            );
        }
    }
}
