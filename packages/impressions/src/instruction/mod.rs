//! Instruction decoding and disassembly.

pub mod operand;
pub mod operation;
pub mod prefix;

mod error;
mod inspect;
mod mnemonic;

use std::fmt::{self, Display};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::SmallVec;

use crate::memory::cursor::{AsCursor, SimpleCursor};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use self::inspect::Inspection;
use self::operation::{Error as OpError, Operation};
use self::prefix::{Prefix, Prefixes};

pub use self::error::Error;
pub use self::mnemonic::Mnemonic;

/// A 32-bit x86 instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    prefixes: Prefixes,
    operation: Operation,
}

impl Instruction {
    /// Gets the prefixes.
    pub const fn prefixes(&self) -> &Prefixes {
        &self.prefixes
    }

    /// Gets the operation.
    pub const fn operation(&self) -> &Operation {
        &self.operation
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        self.operation.mnemonic()
    }
}

impl Extent for Instruction {
    fn size(&self) -> Size {
        Size::new_valid(self.prefixes.len() as u64 + self.operation.size().get())
    }
}

impl Inspect for Instruction {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .identified()
            .label(&self)
            .value(&Inspection(self))
            .finish();
    }
}

impl Encode for Instruction {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        self.prefixes.encode(encoder)?;
        self.operation.encode(encoder)
    }
}

impl Decode for Instruction {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let mut prefixes = Prefixes::new();
        let operation = loop {
            match Operation::decode_with(decoder, &prefixes) {
                Ok(operation) => break operation,
                Err(OpError::InvalidOpcode(byte)) => {
                    prefixes
                        .push(Prefix::try_from(byte).map_err(|_| OpError::InvalidOpcode(byte))?)?;
                }
                Err(err) => return Err(err.into()),
            }
        };

        Ok(Self {
            operation,
            prefixes,
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.operation.fmt_with_prefixes(f, &self.prefixes)
    }
}

impl AsCursor for Instruction {
    type Cursor<'a> = SimpleCursor<'a, Self>;

    fn cursor(&self) -> Self::Cursor<'_> {
        SimpleCursor::new(self)
    }
}

impl Serialize for Instruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error as _;

        let mut bytes = SmallVec::<[u8; 15]>::new();

        self.encode(&mut bytes).map_err(S::Error::custom)?;
        bytes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Instruction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;

        let bytes = SmallVec::<[u8; 15]>::deserialize(deserializer)?;

        if bytes.len() > 15 {
            return Err(D::Error::custom("instruction exceeds 15 bytes"));
        }

        let mut decoder = bytes.as_slice();
        let instruction = Self::decode(&mut decoder).map_err(D::Error::custom)?;

        if !decoder.is_empty() {
            return Err(D::Error::custom("trailing bytes after instruction"));
        }

        Ok(instruction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifts_and_rotates_preserve_prefixes_counts_and_serialization() {
        for (extension, name) in [
            (0, "rol"),
            (1, "ror"),
            (2, "rcl"),
            (3, "rcr"),
            (4, "shl"),
            (5, "shr"),
            (7, "sar"),
        ] {
            for (operand_bytes, destination) in [
                (&b"\xc1"[..], "ecx"),
                (&b"\x44\x8b\xf0"[..], "dword ptr fs:[ebx+ecx*4-16]"),
            ] {
                for count in [0, 1, 8, 31, 32, 33, 128, 255] {
                    let mut bytes = vec![0x64, 0xc1];
                    bytes.extend_from_slice(operand_bytes);
                    bytes[2] |= extension << 3;
                    bytes.push(count);
                    let instruction = Instruction::decode(&mut bytes.as_slice()).unwrap();

                    assert_eq!(
                        instruction.to_string(),
                        format!("{name} {destination}, {count}")
                    );
                    assert_eq!(instruction.size(), bytes.len() as u64);
                    let json = serde_json::to_string(&instruction).unwrap();
                    assert_eq!(json, serde_json::to_string(&bytes).unwrap());
                    assert_eq!(
                        serde_json::from_str::<Instruction>(&json).unwrap(),
                        instruction,
                    );
                    let mut encoded = Vec::new();
                    instruction.encode(&mut encoded).unwrap();
                    assert_eq!(encoded, bytes);
                }
            }
        }
    }

    #[test]
    fn undocumented_shift_extension_remains_unsupported() {
        for bytes in [&b"\xc1\xf0\x08"[..], &b"\x64\xc1\xf0\x08"[..]] {
            assert_eq!(
                Instruction::decode(&mut &bytes[..]),
                Err(Error::Operation(operation::Error::InvalidOpcodeExtension {
                    opcode: 0xc1,
                    extension: 6,
                })),
            );
        }
    }

    #[test]
    fn immediate_groups_preserve_prefixes_and_serialize_all_operations() {
        for (extension, name) in [
            (0, "add"),
            (1, "or"),
            (2, "adc"),
            (3, "sbb"),
            (4, "and"),
            (5, "sub"),
            (6, "xor"),
            (7, "cmp"),
        ] {
            for opcode in [0x81, 0x83] {
                for (operand_bytes, destination) in [
                    (&b"\xc0"[..], "eax"),
                    (&b"\x05\x00\x00\x00\x00"[..], "dword ptr fs:[0x00000000]"),
                ] {
                    let mut bytes = vec![0x64, opcode];

                    bytes.extend_from_slice(operand_bytes);
                    bytes[2] |= extension << 3;

                    if opcode == 0x81 {
                        bytes.extend_from_slice(&(-1i32).to_le_bytes());
                    } else {
                        bytes.push(0xff);
                    }

                    let instruction = Instruction::decode(&mut bytes.as_slice()).unwrap();

                    assert_eq!(instruction.to_string(), format!("{name} {destination}, -1"));
                    assert_eq!(instruction.size(), bytes.len() as u64);

                    let json = serde_json::to_string(&instruction).unwrap();

                    assert_eq!(json, serde_json::to_string(&bytes).unwrap());
                    assert_eq!(
                        serde_json::from_str::<Instruction>(&json).unwrap(),
                        instruction
                    );

                    let mut encoded = Vec::new();

                    instruction.encode(&mut encoded).unwrap();

                    assert_eq!(encoded, bytes);
                }
            }
        }
    }

    #[test]
    fn register_destination_arithmetic_preserves_prefixes_and_serialization() {
        for (opcode, name) in [
            (0x03, "add"),
            (0x0b, "or"),
            (0x13, "adc"),
            (0x1b, "sbb"),
            (0x23, "and"),
            (0x2b, "sub"),
            (0x33, "xor"),
            (0x3b, "cmp"),
        ] {
            for (operand_bytes, operands) in [
                (&b"\xca"[..], "ecx, edx"),
                (&b"\x4c\x8b\xf0"[..], "ecx, fs:[ebx+ecx*4-16]"),
                (&b"\x0d\x38\x81\x5d\x00"[..], "ecx, fs:[0x005d8138]"),
            ] {
                let mut bytes = vec![0x64, opcode];
                bytes.extend_from_slice(operand_bytes);
                let instruction = Instruction::decode(&mut bytes.as_slice()).unwrap();
                assert_eq!(instruction.to_string(), format!("{name} {operands}"));
                assert_eq!(instruction.size(), bytes.len() as u64);
                let json = serde_json::to_string(&instruction).unwrap();
                assert_eq!(json, serde_json::to_string(&bytes).unwrap());
                assert_eq!(
                    serde_json::from_str::<Instruction>(&json).unwrap(),
                    instruction,
                );
                let mut encoded = Vec::new();
                instruction.encode(&mut encoded).unwrap();
                assert_eq!(encoded, bytes);
            }
        }
    }

    #[test]
    fn fs_override_supports_explicit_memory_operands() {
        for (bytes, operation, display) in [
            (
                &b"\x64\xa3\x34\xbf\x60\x01"[..],
                &b"\xa3\x34\xbf\x60\x01"[..],
                "mov fs:[0x0160bf34], eax",
            ),
            (
                &b"\x64\xc1\x64\x8b\xf0\x08"[..],
                &b"\xc1\x64\x8b\xf0\x08"[..],
                "shl dword ptr fs:[ebx+ecx*4-16], 8",
            ),
            (
                &b"\x64\x81\x25\x38\x81\x5d\x00\xff\x00\x00\x00"[..],
                &b"\x81\x25\x38\x81\x5d\x00\xff\x00\x00\x00"[..],
                "and dword ptr fs:[0x005d8138], 255",
            ),
            (
                &b"\x64\x8a\x25\x38\x81\x5d\x00"[..],
                &b"\x8a\x25\x38\x81\x5d\x00"[..],
                "mov ah, fs:[0x005d8138]",
            ),
            (
                &b"\x64\x8a\x64\x8b\xf0"[..],
                &b"\x8a\x64\x8b\xf0"[..],
                "mov ah, fs:[ebx+ecx*4-16]",
            ),
            (
                &b"\x64\x33\x15\x38\x81\x5d\x00"[..],
                &b"\x33\x15\x38\x81\x5d\x00"[..],
                "xor edx, fs:[0x005d8138]",
            ),
            (
                &b"\x64\x33\x54\x8b\xf0"[..],
                &b"\x33\x54\x8b\xf0"[..],
                "xor edx, fs:[ebx+ecx*4-16]",
            ),
            (
                &b"\x64\xff\x15\x38\x81\x5d\x00"[..],
                &b"\xff\x15\x38\x81\x5d\x00"[..],
                "call dword ptr fs:[0x005d8138]",
            ),
            (
                &b"\x64\x89\x25\x00\x00\x00\x00"[..],
                &b"\x89\x25\x00\x00\x00\x00"[..],
                "mov fs:[0x00000000], esp",
            ),
            (
                &b"\x64\x89\x44\x8b\xf0"[..],
                &b"\x89\x44\x8b\xf0"[..],
                "mov fs:[ebx+ecx*4-16], eax",
            ),
            (
                &b"\x64\xa1\x00\x00\x00\x00"[..],
                &b"\xa1\x00\x00\x00\x00"[..],
                "mov eax, fs:[0x00000000]",
            ),
            (&b"\x64\x8b\x03"[..], &b"\x8b\x03"[..], "mov eax, fs:[ebx]"),
            (
                &b"\x64\x8b\x44\x8b\xf0"[..],
                &b"\x8b\x44\x8b\xf0"[..],
                "mov eax, fs:[ebx+ecx*4-16]",
            ),
        ] {
            let instruction = Instruction::decode(&mut &bytes[..]).unwrap();
            let unprefixed = Instruction::decode(&mut &operation[..]).unwrap();

            assert_eq!(instruction.operation(), unprefixed.operation());
            assert!(instruction.prefixes().has_fs_override());
            assert_eq!(instruction.prefixes().len(), 1);
            assert_eq!(instruction.to_string(), display);
            assert_eq!(instruction.size(), bytes.len() as u64);

            let mut encoded = Vec::new();

            instruction.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);

            let json = serde_json::to_string(&instruction).unwrap();

            assert_eq!(json, serde_json::to_string(bytes).unwrap());
            assert_eq!(
                serde_json::from_str::<Instruction>(&json).unwrap(),
                instruction
            );
        }
    }

    #[test]
    fn fs_override_does_not_change_register_or_stack_operands() {
        for (bytes, display) in [
            (&b"\x64\xc1\xe1\x08"[..], "shl ecx, 8"),
            (&b"\x64\x81\xe1\xff\x00\x00\x00"[..], "and ecx, 255"),
            (&b"\x64\x8a\xd4"[..], "mov dl, ah"),
            (&b"\x64\x33\xd2"[..], "xor edx, edx"),
            (&b"\x64\xff\xd0"[..], "call eax"),
            (&b"\x64\x89\xc1"[..], "mov ecx, eax"),
            (&b"\x64\x8b\xc1"[..], "mov eax, ecx"),
            (&b"\x64\x53"[..], "push ebx"),
            (&b"\x64\x6a\xff"[..], "push -1"),
            (&b"\x64\x68\xff\xff\xff\xff"[..], "push -1"),
        ] {
            let instruction = Instruction::decode(&mut &bytes[..]).unwrap();

            assert_eq!(instruction.to_string(), display);

            let mut encoded = Vec::new();

            instruction.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }
    }

    #[test]
    fn fs_override_preserves_repetitions_within_prefix_capacity() {
        for count in 1..=4 {
            let mut bytes = vec![0x64; count];

            bytes.extend_from_slice(b"\xa1\x00\x00\x00\x00");

            let instruction = Instruction::decode(&mut bytes.as_slice()).unwrap();

            assert_eq!(instruction.prefixes().len(), count);
            assert_eq!(instruction.size(), bytes.len() as u64);

            let mut encoded = Vec::new();

            instruction.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }

        assert_eq!(
            Instruction::decode(&mut &b"\x64\x64\x64\x64\x64\xa1\x00\x00\x00\x00"[..]),
            Err(Error::Prefix(prefix::Error::TooManyPrefixes)),
        );
    }

    #[test]
    fn fs_override_requires_complete_operation() {
        let bytes = b"\x64\xa1\x00\x00\x00\x00";

        for length in 1..bytes.len() {
            assert!(matches!(
                Instruction::decode(&mut &bytes[..length]),
                Err(Error::Operation(operation::Error::Decode(_))),
            ));
        }
    }

    #[test]
    fn unsupported_opcodes_remain_operation_errors() {
        for bytes in [&b"\x90"[..], &b"\x64\x90"[..]] {
            assert_eq!(
                Instruction::decode(&mut &bytes[..]),
                Err(Error::Operation(operation::Error::InvalidOpcode(0x90))),
            );
        }
    }

    #[test]
    fn serde_round_trips_every_push_register_encoding() {
        for opcode in 0x50..=0x57 {
            let bytes = [opcode];
            let instruction = Instruction::decode(&mut &bytes[..]).unwrap();
            let json = serde_json::to_string(&instruction).unwrap();

            assert_eq!(json, format!("[{opcode}]"));

            let restored = serde_json::from_str::<Instruction>(&json).unwrap();

            assert_eq!(restored, instruction);

            let mut encoded = SmallVec::<[u8; 15]>::new();

            restored.encode(&mut encoded).unwrap();

            assert_eq!(encoded.as_slice(), bytes);
            assert!(!encoded.spilled());
        }
    }

    #[test]
    fn serde_includes_opcode_and_operand_bytes() {
        for (json, display) in [
            ("[163,52,191,96,1]", "mov [0x0160bf34], eax"),
            ("[3,202]", "add ecx, edx"),
            ("[193,225,8]", "shl ecx, 8"),
            ("[193,225,255]", "shl ecx, 255"),
            ("[129,225,255,0,0,0]", "and ecx, 255"),
            ("[129,225,1,0,0,0]", "and ecx, 1"),
            ("[138,212]", "mov dl, ah"),
            ("[138,100,139,240]", "mov ah, [ebx+ecx*4-16]"),
            ("[51,210]", "xor edx, edx"),
            ("[51,84,139,240]", "xor edx, [ebx+ecx*4-16]"),
            ("[255,21,56,129,93,0]", "call dword ptr [0x005d8138]"),
            ("[255,208]", "call eax"),
            ("[131,236,88]", "sub esp, 88"),
            ("[131,232,255]", "sub eax, -1"),
            ("[100,131,45,0,0,0,0,1]", "sub dword ptr fs:[0x00000000], 1"),
            ("[100,131,236,88]", "sub esp, 88"),
            ("[83]", "push ebx"),
            ("[106,255]", "push -1"),
            ("[106,128]", "push -128"),
            ("[106,0]", "push 0"),
            ("[106,127]", "push 127"),
            ("[104,255,255,255,255]", "push -1"),
            ("[104,0,0,0,128]", "push -2147483648"),
            ("[104,255,255,255,127]", "push 2147483647"),
            ("[104,120,86,52,18]", "push 305419896"),
            ("[104,0,0,0,0]", "push 0"),
            ("[139,193]", "mov eax, ecx"),
            ("[139,68,139,240]", "mov eax, [ebx+ecx*4-16]"),
            ("[161,0,0,0,0]", "mov eax, [0x00000000]"),
            ("[161,120,86,52,18]", "mov eax, [0x12345678]"),
            ("[161,255,255,255,255]", "mov eax, [0xffffffff]"),
            ("[100,161,0,0,0,0]", "mov eax, fs:[0x00000000]"),
        ] {
            let instruction = serde_json::from_str::<Instruction>(json).unwrap();

            assert_eq!(instruction.to_string(), display);
            assert_eq!(serde_json::to_string(&instruction).unwrap(), json);
        }
    }

    #[test]
    fn serde_rejects_invalid_or_incomplete_input() {
        for json in [
            "[129]",
            "[129,225,255,0,0]",
            "[138]",
            "[138,37,56,129,93]",
            "[51]",
            "[51,21,56,129,93]",
            "[255]",
            "[255,21,56,129,93]",
            "[255,192]",
            "[131]",
            "[131,236]",
            "[]",
            "[-1]",
            "[139]",
            "[106]",
            "[161]",
            "[161,0]",
            "[161,0,0]",
            "[161,0,0,0]",
            "[104]",
            "[104,1]",
            "[104,1,2]",
            "[104,1,2,3]",
            "[139,4]",
            "[139,4,37]",
            "[139,67]",
            "[139,131,1,2,3]",
            "[144]",
            "[139,67,256]",
            "[139,67,-1]",
            "[256]",
        ] {
            assert!(serde_json::from_str::<Instruction>(json).is_err(), "{json}");
        }
    }

    #[test]
    fn serde_rejects_instructions_longer_than_fifteen_bytes() {
        let json = serde_json::to_string(&[0x53; 16]).unwrap();
        let err = serde_json::from_str::<Instruction>(&json).unwrap_err();

        assert!(err.to_string().contains("instruction exceeds 15 bytes"));
    }

    #[test]
    fn serde_rejects_trailing_bytes() {
        let err = serde_json::from_str::<Instruction>("[83,80]").unwrap_err();

        assert!(err.to_string().contains("trailing bytes after instruction"));
    }

    #[test]
    fn serde_preserves_zero_displacement_widths() {
        for bytes in [
            &b"\x8b\x03"[..],
            &b"\x8b\x43\x00"[..],
            &b"\x8b\x83\x00\x00\x00\x00"[..],
            &b"\x8b\x04\xe3"[..],
        ] {
            let instruction = Instruction::decode(&mut &bytes[..]).unwrap();
            let json = serde_json::to_string(&instruction).unwrap();

            assert_eq!(json, serde_json::to_string(bytes).unwrap());

            let restored = serde_json::from_str::<Instruction>(&json).unwrap();

            assert_eq!(restored, instruction);

            let mut encoded = Vec::new();

            restored.encode(&mut encoded).unwrap();

            assert_eq!(encoded, bytes);
        }
    }
}
