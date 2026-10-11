//! Instruction decoding and disassembly.

pub mod operand;
pub mod operation;

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
use self::operation::Operation;

pub use self::error::Error;
pub use self::mnemonic::Mnemonic;

/// A 32-bit x86 instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    operation: Operation,
}

impl Instruction {
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
        self.operation.size()
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
        self.operation.encode(encoder)
    }
}

impl Decode for Instruction {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            operation: Operation::decode(decoder)?,
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.operation, f)
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
    fn serde_rejects_invalid_or_incomplete_input() {
        for json in ["[]", "[144]", "[-1]", "[256]"] {
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
}
