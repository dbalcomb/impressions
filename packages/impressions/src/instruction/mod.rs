//! Instruction decoding and disassembly.

pub mod operand;
pub mod operation;

mod error;
mod inspect;
mod mnemonic;

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
